use std::collections::HashSet;
use std::sync::OnceLock;

use crate::config::{Config, UseProfiles};
use crate::lists as l;

pub enum SetRef<'a> {
    Static(&'static HashSet<String>),
    Owned(HashSet<String>, std::marker::PhantomData<&'a ()>),
}

impl SetRef<'_> {
    pub fn contains(&self, name: &str) -> bool {
        match self {
            SetRef::Static(set) => set.contains(name),
            SetRef::Owned(set, _) => set.contains(name),
        }
    }
}

fn default_tags() -> &'static HashSet<String> {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    SET.get_or_init(|| {
        l::TAGS_HTML
            .iter()
            .chain(l::TAGS_SVG)
            .chain(l::TAGS_SVG_FILTERS)
            .chain(l::TAGS_MATHML)
            .map(|s| (*s).to_owned())
            .collect()
    })
}

fn default_attrs() -> &'static HashSet<String> {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    SET.get_or_init(|| {
        l::ATTRS_HTML
            .iter()
            .chain(l::ATTRS_SVG)
            .chain(l::ATTRS_MATHML)
            .chain(l::ATTRS_XML)
            .map(|s| (*s).to_owned())
            .collect()
    })
}

