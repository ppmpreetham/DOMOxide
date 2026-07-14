use crate::hooks::Hooks;
use serde::{Deserialize, Serialize};

/// dompurify's USE_PROFILES
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseProfiles {
    #[serde(default)]
    pub html: bool,
    #[serde(default)]
    pub svg: bool,
    #[serde(default, rename = "svgFilters")]
    pub svg_filters: bool,
    #[serde(default, rename = "mathMl")]
    pub math_ml: bool,
}

impl UseProfiles {
    /// USE_PROFILES: { html: true }
    pub fn html() -> Self {
        Self {
            html: true,
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CustomElementHandling {
    #[serde(default, rename = "tagNameCheck")]
    pub allow_custom_elements: bool,
    #[serde(default, rename = "allowCustomizedBuiltInElements")]
    pub allow_customized_builtins: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(
        default,
        rename = "ALLOWED_TAGS",
        skip_serializing_if = "Option::is_none"
    )]
    pub allowed_tags: Option<Vec<String>>,
    #[serde(default, rename = "ADD_TAGS", skip_serializing_if = "Vec::is_empty")]
    pub add_tags: Vec<String>,
    #[serde(default, rename = "FORBID_TAGS", skip_serializing_if = "Vec::is_empty")]
    pub forbid_tags: Vec<String>,
    #[serde(
        default,
        rename = "ALLOWED_ATTR",
        skip_serializing_if = "Option::is_none"
    )]
    pub allowed_attr: Option<Vec<String>>,
    #[serde(default, rename = "ADD_ATTR", skip_serializing_if = "Vec::is_empty")]
    pub add_attr: Vec<String>,
    #[serde(default, rename = "FORBID_ATTR", skip_serializing_if = "Vec::is_empty")]
    pub forbid_attr: Vec<String>,
    #[serde(
        default,
        rename = "USE_PROFILES",
        skip_serializing_if = "Option::is_none"
    )]
    pub use_profiles: Option<UseProfiles>,
    #[serde(
        default,
        rename = "CUSTOM_ELEMENT_HANDLING",
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_element_handling: Option<CustomElementHandling>,
    #[serde(default = "default_true", rename = "ALLOW_DATA_ATTR")]
    pub allow_data_attr: bool,
    #[serde(default = "default_true", rename = "ALLOW_ARIA_ATTR")]
    pub allow_aria_attr: bool,
    #[serde(default = "default_true", rename = "SANITIZE_DOM")]
    pub sanitize_dom: bool,
    #[serde(default = "default_true", rename = "SAFE_FOR_XML")]
    pub safe_for_xml: bool,
    #[serde(default = "default_true", rename = "KEEP_CONTENT")]
    pub keep_content: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            allowed_tags: None,
            add_tags: Vec::new(),
            forbid_tags: Vec::new(),
            allowed_attr: None,
            add_attr: Vec::new(),
            forbid_attr: Vec::new(),
            use_profiles: None,
            custom_element_handling: None,
            allow_data_attr: true,
            allow_aria_attr: true,
            sanitize_dom: true,
            safe_for_xml: true,
            keep_content: true,
        }
    }
}

impl Config {
    pub fn html_profile() -> Self {
        Self {
            use_profiles: Some(UseProfiles::html()),
            ..Self::default()
        }
    }

    pub fn add_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.add_tags.extend(tags.into_iter().map(Into::into));
        self
    }

    pub fn add_attr<I, S>(mut self, attrs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.add_attr.extend(attrs.into_iter().map(Into::into));
        self
    }

    pub fn forbid_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.forbid_tags.extend(tags.into_iter().map(Into::into));
        self
    }

    pub fn forbid_attr<I, S>(mut self, attrs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.forbid_attr.extend(attrs.into_iter().map(Into::into));
        self
    }

    pub fn clean(&self, dirty: &str) -> String {
        crate::sanitize::clean(dirty, &crate::policy::Policy::new(self), &Hooks::default())
    }
}
