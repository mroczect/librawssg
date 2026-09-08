use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct ContentRule {
    pub name: String,
    pub pattern: String,
    pub template: String,
    #[serde(default)]
    pub list_template: Option<String>,
    #[serde(default)]
    pub list_enabled: bool,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl ContentRule {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        pattern: impl Into<String>,
        template: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            pattern: pattern.into(),
            template: template.into(),
            list_template: None,
            list_enabled: false,
            extra: HashMap::new(),
        }
    }
}
