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

