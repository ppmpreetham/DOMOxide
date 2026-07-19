use std::cell::RefCell;
use std::collections::HashMap;

use html5ever::tendril::TendrilSink;

use crate::attributes::{self, is_basic_custom_element};
use crate::dom::{self, Handle, NodeData};
use crate::escape::{
    escape_attribute, escape_text, has_mxss_marker, has_noscript_breakout, is_raw_text,
};
use crate::hooks::{HookAction, Hooks};
use crate::lists as l;
use crate::policy::Policy;
use crate::preprocess;

/// html parser produces ts
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ns {
    Html,
    Svg,
    MathMl,
}

impl Ns {
    fn from_url(url: &str) -> Self {
        match url {
            "http://www.w3.org/2000/svg" => Ns::Svg,
            "http://www.w3.org/1998/Math/MathML" => Ns::MathMl,
            _ => Ns::Html,
        }
    }

    /// namespace declaration url browsers keep on `<svg>`/`<math>`.
    fn xmlns_url(self) -> &'static str {
        match self {
            Ns::Svg => "http://www.w3.org/2000/svg",
            Ns::MathMl => "http://www.w3.org/1998/Math/MathML",
            Ns::Html => "http://www.w3.org/1999/xhtml",
        }
    }
}

/// dirty -> clean html
pub(crate) fn clean(dirty: &str, policy: &Policy, hooks: &Hooks) -> String {
    if !dirty.contains('<') {
        return dirty.to_owned();
    }

    let scan = preprocess::prescan(dirty);
    let dirty = scan.html.as_deref().unwrap_or(dirty);

    let parse_opts = html5ever::ParseOpts {
        tree_builder: html5ever::tree_builder::TreeBuilderOpts {
            drop_doctype: true,
            scripting_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = html5ever::parse_document(dom::RcDom::default(), parse_opts).one(dirty);

    let mut out = String::with_capacity(dirty.len());
    if let Some(body) = find_body(&dom.document) {
        escape_text(leading_whitespace(dirty), &mut out);
        let children = children_of(&body);
        let mut walker = Walker {
            policy,
            hooks,
            scratch: String::new(),
            xmlns_spots: &scan.xmlns_spots,
            foreign_seen: 0,
        };
        walker.children(&children, Ns::Html, "body", &mut out);
    }
    out
}

/// `^[\r\n\t ]+` prefix that parsers drop but we re-insert.
fn leading_whitespace(dirty: &str) -> &str {
    let end = dirty
        .find(|c: char| !matches!(c, '\r' | '\n' | '\t' | ' '))
        .unwrap_or(dirty.len());
    &dirty[..end]
}

/// locates `html > body`
fn find_body(document: &Handle) -> Option<Handle> {
    let top = document.children.borrow();
    let html = top.iter().find(|child| elem_name(child) == Some("html"))?;
    let nested = html.children.borrow();
    if nested
        .iter()
        .any(|child| elem_name(child) == Some("frameset"))
    {
        return None;
    }
    nested
        .iter()
        .find(|child| elem_name(child) == Some("body"))
        .cloned()
}

/// template contents live in a separate fragment; everything else in place.
fn children_of(node: &Handle) -> Vec<Handle> {
    if let NodeData::Element {
        template_contents, ..
    } = &node.data
        && let Some(fragment) = template_contents.borrow().as_ref()
    {
        return fragment.children.borrow().clone();
    }

    node.children.borrow().clone()
}

fn elem_name(node: &Handle) -> Option<&str> {
    match &node.data {
        NodeData::Element { name, .. } => Some(name.local.as_ref()),
        _ => None,
    }
}

struct Walker<'a> {
    policy: &'a Policy,
    hooks: &'a Hooks,
    scratch: String,
    /// (nth svg/math tag, xmlns attr position) recorded by the prescan
    xmlns_spots: &'a [(u32, u16)],
    foreign_seen: u32,
}

impl Walker<'_> {
    fn children(&mut self, nodes: &[Handle], parent_ns: Ns, parent_tag: &str, out: &mut String) {
        for node in nodes {
            self.node(node, parent_ns, parent_tag, out);
        }
    }

    fn node(&mut self, node: &Handle, parent_ns: Ns, parent_tag: &str, out: &mut String) {
        match &node.data {
            NodeData::Text { contents } => {
                let text = contents.borrow();
                if is_raw_text(parent_tag) {
                    out.push_str(&text);
                } else {
                    escape_text(&text, out);
                }
            }
            // comments and processing instructions are not allow-listed: dropped.
            NodeData::Comment { .. }
            | NodeData::Doctype { .. }
            | NodeData::ProcessingInstruction { .. }
            | NodeData::Document => {}
            NodeData::Element { .. } => self.element(node, parent_ns, parent_tag, out),
        }
    }

    fn element(&mut self, node: &Handle, parent_ns: Ns, parent_tag: &str, out: &mut String) {
        let NodeData::Element { name, attrs, .. } = &node.data else {
            unreachable!("element() called on a non-element");
        };
        let tag = name.local.to_string();
        let tag_lower = tag.to_lowercase();
        let ns = Ns::from_url(name.ns.as_ref());
        let mut attr_list = parse_attrs(attrs);
        self.restore_xmlns(&mut attr_list, ns, &tag_lower);
        let children = children_of(node);

        if self.removed_by_element_hooks(&tag_lower, &attr_list) {
            return;
        }

        // SAFE_FOR_XML mXss probe on text-only subtrees; rawtext parents
        // serialize their text unescaped, which is what the probe must see.
        if self.policy.safe_for_xml
            && !children.is_empty()
            && !children
                .iter()
                .any(|child| matches!(child.data, NodeData::Element { .. }))
            && has_mxss_marker(&raw_inner_html(&children, is_raw_text(&tag_lower)))
            && has_mxss_marker(&raw_text_of(&children))
        {
            return;
        }
        // style elements must not contain element children (html namespace).
        if ns == Ns::Html
            && tag_lower == "style"
            && children
                .iter()
                .any(|child| matches!(child.data, NodeData::Element { .. }))
        {
            return;
        }

        let forbidden = self.policy.forbids_tag(&tag_lower);
        let custom_ok = !forbidden && self.is_custom_tag(&tag_lower);
        if forbidden || !(self.policy.allows_tag(&tag_lower) || custom_ok) {
            if self.policy.keep_content && !self.policy.drops_content_of(&tag_lower) {
                self.children(&children, parent_ns, parent_tag, out);
            }
            return;
        }

        if !namespace_valid(ns, &tag_lower, parent_ns, parent_tag)
            || matches!(tag_lower.as_str(), "noscript" | "noembed" | "noframes")
                && has_noscript_breakout(&raw_inner_html(&children, false))
        {
            return;
        }

        let sanitized = attributes::sanitize(
            self.policy,
            self.hooks,
            &tag_lower,
            attr_list,
            &mut self.scratch,
        );

        out.push('<');
        out.push_str(&tag);
        for attr in &sanitized {
            out.push(' ');
            out.push_str(&attr.name);
            out.push_str("=\"");
            escape_attribute(&attr.value, out);
            out.push('"');
        }
        out.push('>');

        if self
            .hooks
            .after_sanitize_elements
            .as_ref()
            .is_some_and(|hook| hook(&tag_lower, &mut HashMap::new()) == HookAction::ForceRemove)
        {
            return;
        }

        // void elements serialize without an end tag or children.
        if l::VOID_ELEMENTS.contains(&tag_lower.as_str()) {
            return;
        }
        self.children(&children, ns, &tag_lower, out);
        out.push_str("</");
        out.push_str(&tag);
        out.push('>');
    }

    /// re-inserts the `xmlns` declaration html5ever consumed when the source
    /// carried one, at its original attribute position.
    fn restore_xmlns(&mut self, attr_list: &mut Vec<ParsedAttr>, ns: Ns, tag_lower: &str) {
        if !matches!((ns, tag_lower), (Ns::Svg, "svg") | (Ns::MathMl, "math")) {
            return;
        }
        let ordinal = self.foreign_seen;
        self.foreign_seen += 1;
        let Some((_, pos)) = self.xmlns_spots.iter().find(|(ord, _)| *ord == ordinal) else {
            return;
        };
        let spot = ParsedAttr {
            orig: *pos as usize,
            name: "xmlns".to_owned(),
            value: ns.xmlns_url().to_owned(),
        };
        let insert_at = attr_list.partition_point(|attr| attr.orig <= *pos as usize);
        attr_list.insert(insert_at.min(attr_list.len()), spot);
    }

    /// before/uponSanitizeElement hooks may force-remove the element.
    fn removed_by_element_hooks(&self, tag_lower: &str, attrs: &[ParsedAttr]) -> bool {
        let run = |hook: Option<&Box<crate::hooks::ElementHook>>| {
            hook.is_some_and(|hook| {
                hook(
                    tag_lower,
                    &mut attrs
                        .iter()
                        .map(|a| (a.name.clone(), a.value.clone()))
                        .collect(),
                ) == HookAction::ForceRemove
            })
        };
        run(self.hooks.before_sanitize_elements.as_ref())
            || run(self.hooks.upon_sanitize_element.as_ref())
    }

    /// CUSTOM_ELEMENT_HANDLING gate for well-formed custom names.
    fn is_custom_tag(&self, tag_lower: &str) -> bool {
        self.policy
            .custom
            .is_some_and(|custom| custom.allow_custom_elements)
            && is_basic_custom_element(tag_lower)
    }
}

type AttrVec = RefCell<Vec<html5ever::Attribute>>;

/// an attribute as parsed, remembering its source position.
pub(crate) struct ParsedAttr {
    pub(crate) orig: usize,
    pub(crate) name: String,
    pub(crate) value: String,
}

fn parse_attrs(attrs: &AttrVec) -> Vec<ParsedAttr> {
    attrs
        .borrow()
        .iter()
        .enumerate()
        .map(|(orig, attr)| ParsedAttr {
            orig,
            name: attr_name(attr),
            value: attr.value.to_string(),
        })
        .collect()
}

/// renders prefixed attribute names exactly like dom serialization does.
fn attr_name(attr: &html5ever::Attribute) -> String {
    use html5ever::ns;
    let local = attr.name.local.as_ref();
    match attr.name.prefix.as_deref() {
        Some(prefix) => format!("{prefix}:{local}"),
        None if attr.name.ns == ns!(xlink) => format!("xlink:{local}"),
        None if attr.name.ns == ns!(xml) => format!("xml:{local}"),
        None if attr.name.ns == ns!(xmlns) && local != "xmlns" => format!("xmlns:{local}"),
        None => local.to_owned(),
    }
}

