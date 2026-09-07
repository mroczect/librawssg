use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SiteConfig {
    #[serde(default)]
    pub navbar: Vec<super::NavItem>,
    #[serde(default)]
    pub sidebar: Vec<super::NavItem>,
    #[serde(default = "default_site_name")]
    pub site_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_language")]
    pub language: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_site_name() -> String {
    "librawssg".into()
}

#[allow(clippy::unnecessary_wraps)]
fn default_language() -> Option<String> {
    Some("en".into())
}

impl SiteConfig {
    #[must_use]
    pub fn new(site_name: impl Into<String>) -> Self {
        Self {
            site_name: site_name.into(),
            ..Self::default()
        }
    }
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            navbar: Vec::new(),
            sidebar: Vec::new(),
            site_name: default_site_name(),
            description: None,
            language: default_language(),
            base_url: None,
            author: None,
            repo_url: None,
            license: None,
            extra: HashMap::new(),
        }
    }
}
