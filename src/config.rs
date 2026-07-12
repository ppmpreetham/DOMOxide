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
