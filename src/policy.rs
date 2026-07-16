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

pub struct Policy {
    tags: SetRef<'static>,
    attrs: SetRef<'static>,
    forbid_tags: Vec<String>,
    forbid_attr: Vec<String>,
    pub(crate) custom: Option<crate::config::CustomElementHandling>,
    pub(crate) allow_data_attr: bool,
    pub(crate) allow_aria_attr: bool,
    pub(crate) sanitize_dom: bool,
    pub(crate) safe_for_xml: bool,
    pub(crate) keep_content: bool,
}

impl Policy {
    pub fn new(config: &Config) -> Self {
        let profiles = config.use_profiles.unwrap_or_default();
        let has_tag_overrides = config.allowed_tags.is_some()
            || !config.add_tags.is_empty()
            || !config.forbid_tags.is_empty()
            || config.use_profiles.is_some();
        let has_attr_overrides = config.allowed_attr.is_some()
            || !config.add_attr.is_empty()
            || !config.forbid_attr.is_empty()
            || config.use_profiles.is_some();

        if !has_tag_overrides && !has_attr_overrides {
            return Self::with_sets(
                SetRef::Static(default_tags()),
                SetRef::Static(default_attrs()),
                config,
                Vec::new(),
                Vec::new(),
            );
        }

        let tags = Self::resolve_tags(config, profiles, has_tag_overrides);
        let attrs = Self::resolve_attrs(config, profiles, has_attr_overrides);

        Self::with_sets(
            SetRef::Owned(tags, std::marker::PhantomData),
            SetRef::Owned(attrs, std::marker::PhantomData),
            config,
            config.forbid_tags.clone(),
            config.forbid_attr.clone(),
        )
    }

    fn resolve_tags(config: &Config, profiles: UseProfiles, overridden: bool) -> HashSet<String> {
        let mut tags = if !overridden {
            default_tags().clone()
        } else if let Some(allowed) = &config.allowed_tags {
            allowed.iter().cloned().collect()
        } else if config.use_profiles.is_some() {
            profile_tags(profiles)
        } else {
            default_tags().clone()
        };
        tags.extend(config.add_tags.iter().cloned());
        for tag in &config.forbid_tags {
            tags.remove(tag);
        }
        if tags.contains("table") {
            tags.insert("tbody".into());
        }
        tags
    }

    fn resolve_attrs(config: &Config, profiles: UseProfiles, overridden: bool) -> HashSet<String> {
        let mut attrs = if !overridden {
            default_attrs().clone()
        } else if let Some(allowed) = &config.allowed_attr {
            allowed.iter().cloned().collect()
        } else if config.use_profiles.is_some() {
            profile_attrs(profiles)
        } else {
            default_attrs().clone()
        };
        attrs.extend(config.add_attr.iter().cloned());
        for attr in &config.forbid_attr {
            attrs.remove(attr);
        }
        attrs
    }

    fn with_sets(
        tags: SetRef<'static>,
        attrs: SetRef<'static>,
        config: &Config,
        forbid_tags: Vec<String>,
        forbid_attr: Vec<String>,
    ) -> Self {
        Self {
            tags,
            attrs,
            forbid_tags,
            forbid_attr,
            custom: config.custom_element_handling,
            allow_data_attr: config.allow_data_attr,
            allow_aria_attr: config.allow_aria_attr,
            sanitize_dom: config.sanitize_dom,
            safe_for_xml: config.safe_for_xml,
            keep_content: config.keep_content,
        }
    }

    pub fn allows_tag(&self, tag_lower: &str) -> bool {
        self.tags.contains(tag_lower)
    }

    pub fn forbids_tag(&self, tag_lower: &str) -> bool {
        self.forbid_tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case(tag_lower))
    }

    pub fn allows_attr(&self, attr_lower: &str) -> bool {
        self.attrs.contains(attr_lower)
    }

    pub fn forbids_attr(&self, attr_lower: &str) -> bool {
        self.forbid_attr
            .iter()
            .any(|a| a.eq_ignore_ascii_case(attr_lower))
    }

    pub fn drops_content_of(&self, tag_lower: &str) -> bool {
        l::FORBID_CONTENTS.contains(&tag_lower)
    }

    pub fn is_known_svg_tag(tag_lower: &str) -> bool {
        static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
        SET.get_or_init(|| {
            l::TAGS_SVG
                .iter()
                .chain(l::TAGS_SVG_FILTERS)
                .chain(l::TAGS_SVG_DISALLOWED)
                .copied()
                .collect()
        })
        .contains(tag_lower)
    }

    pub fn is_known_mathml_tag(tag_lower: &str) -> bool {
        static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
        SET.get_or_init(|| {
            l::TAGS_MATHML
                .iter()
                .chain(l::TAGS_MATHML_DISALLOWED)
                .copied()
                .collect()
        })
        .contains(tag_lower)
    }
}
