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
