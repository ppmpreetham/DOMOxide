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
