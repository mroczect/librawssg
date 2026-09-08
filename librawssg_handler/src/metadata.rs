use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct Metadata {
    pub title: String,
    pub description: String,
    pub author: Option<String>,
    pub repo_url: Option<String>,
    pub license: Option<String>,
    pub date: Option<NaiveDate>,
    pub updated: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub draft: bool,
    pub extra: HashMap<String, serde_json::Value>,
}

impl Metadata {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> librawssg_error::Result<Self> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(librawssg_error::Error::Validation(
                "metadata title cannot be empty".into(),
            ));
        }
        Ok(Self {
            title,
            description: description.into(),
            ..Default::default()
        })
    }

    #[must_use]
    pub const fn is_draft(&self) -> bool {
        self.draft
    }

    pub fn insert_extra(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        let _ = self.extra.insert(key.into(), value.into());
    }

    #[must_use]
    pub fn get_extra(&self, key: &str) -> Option<&serde_json::Value> {
        self.extra.get(key)
    }
}
