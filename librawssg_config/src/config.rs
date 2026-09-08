use librawssg_error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{BuildConfig, ContentRule, SiteConfig};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct Config {
    pub site: SiteConfig,
    pub build: BuildConfig,
    pub content_rules: Vec<ContentRule>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_site_name(mut self, name: impl Into<String>) -> Self {
        self.site.site_name = name.into();
        self
    }

    pub fn add_content_rule(&mut self, rule: ContentRule) {
        self.content_rules.push(rule);
    }

    #[must_use]
    pub fn find_rule_by_name(&self, name: &str) -> Option<&ContentRule> {
        self.content_rules.iter().find(|rule| rule.name == name)
    }

    pub fn remove_rule_by_name(&mut self, name: &str) -> Option<ContentRule> {
        let pos = self.content_rules.iter().position(|rule| rule.name == name);
        pos.map(|idx| self.content_rules.remove(idx))
    }

    #[must_use]
    pub fn has_duplicate_rule_names(&self) -> bool {
        let mut seen = std::collections::HashSet::new();
        self.content_rules
            .iter()
            .any(|rule| !seen.insert(rule.name.clone()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.site.site_name.trim().is_empty() {
            return Err(Error::Validation("site_name cannot be empty".into()));
        }

        if self.content_rules.is_empty() {
            return Err(Error::Validation(
                "at least one content rule must be defined".into(),
            ));
        }

        if self.has_duplicate_rule_names() {
            return Err(Error::Validation(
                "duplicate content rule names are not allowed".into(),
            ));
        }

        for (idx, rule) in self.content_rules.iter().enumerate() {
            if rule.name.trim().is_empty() {
                return Err(Error::Validation(format!(
                    "content rule #{idx}: name cannot be empty"
                )));
            }
            if rule.pattern.trim().is_empty() {
                return Err(Error::Validation(format!(
                    "content rule '{}': pattern cannot be empty",
                    rule.name
                )));
            }
            if rule.pattern.contains("..") {
                return Err(Error::Validation(format!(
                    "content rule '{}': pattern contains invalid '..'",
                    rule.name
                )));
            }
            if rule.template.trim().is_empty() {
                return Err(Error::Validation(format!(
                    "content rule '{}': template cannot be empty",
                    rule.name
                )));
            }
        }

        if let Some(base_url) = &self.site.base_url
            && !(base_url.starts_with("http://") || base_url.starts_with("https://"))
        {
            return Err(Error::Validation(
                "site.base_url must start with http:// or https://".into(),
            ));
        }

        Ok(())
    }

    pub fn from_yaml_str(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).map_err(|e| Error::Config(format!("invalid YAML config: {e}")))
    }

    pub fn to_yaml_string(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| Error::Serialization(format!("failed to serialize config: {e}")))
    }

    pub fn from_json_str(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::Config(format!("invalid JSON config: {e}")))
    }

    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| Error::Serialization(format!("failed to serialize config: {e}")))
    }
}
