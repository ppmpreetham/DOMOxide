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

fn profile_tags(
    crate::config::UseProfiles {
        html,
        svg,
        svg_filters,
        math_ml,
    }: crate::config::UseProfiles,
) -> HashSet<String> {
    let mut set = HashSet::new();
    if html {
        set.extend(l::TAGS_HTML.iter().map(|s| (*s).to_owned()));
    }
    if svg {
        set.extend(l::TAGS_SVG.iter().map(|s| (*s).to_owned()));
    }
    if svg_filters {
        set.extend(l::TAGS_SVG_FILTERS.iter().map(|s| (*s).to_owned()));
    }
    if math_ml {
        set.extend(l::TAGS_MATHML.iter().map(|s| (*s).to_owned()));
    }
    set
}

fn profile_attrs(
    crate::config::UseProfiles {
        html,
        svg,
        svg_filters,
        math_ml,
    }: crate::config::UseProfiles,
) -> HashSet<String> {
    let mut set = HashSet::new();
    if html {
        set.extend(l::ATTRS_HTML.iter().map(|s| (*s).to_owned()));
    }
    if svg || svg_filters {
        set.extend(l::ATTRS_SVG.iter().map(|s| (*s).to_owned()));
    }
    if math_ml {
        set.extend(l::ATTRS_MATHML.iter().map(|s| (*s).to_owned()));
    }
    if svg || svg_filters || math_ml {
        set.extend(l::ATTRS_XML.iter().map(|s| (*s).to_owned()));
    }
    set
}

