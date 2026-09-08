use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct NavItem {
    pub label: String,
    pub url: String,
    pub children: Vec<Self>,
}

impl NavItem {
    #[must_use]
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            url: url.into(),
            children: Vec::new(),
        }
    }
}
