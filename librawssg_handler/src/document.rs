use crate::Metadata;
use librawssg_error::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Document {
    pub metadata: Metadata,
    pub body: String,
    pub url: String,
    pub output_path: PathBuf,
    pub source_path: PathBuf,
    pub depth: usize,
    pub content_type: String,
    pub is_list: bool,
    pub list_items: Option<Vec<Self>>,
    pub taxonomies: HashMap<String, Vec<String>>,
}

impl Document {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        metadata: Metadata,
        body: impl Into<String>,
        url: impl Into<String>,
        output_path: impl Into<PathBuf>,
        source_path: impl Into<PathBuf>,
        depth: usize,
        content_type: impl Into<String>,
        is_list: bool,
    ) -> Result<Self> {
        let url = url.into();
        if url.trim().is_empty() {
            return Err(Error::Validation("document url cannot be empty".into()));
        }
        let output_path = output_path.into();
        if output_path.as_os_str().is_empty() {
            return Err(Error::Validation(
                "document output_path cannot be empty".into(),
            ));
        }
        let source_path = source_path.into();
        if source_path.file_name().is_none() {
            return Err(Error::Validation(
                "document source_path must have a file name".into(),
            ));
        }
        if depth > 1000 {
            return Err(Error::Validation(
                "document depth is unreasonably large".into(),
            ));
        }
        Ok(Self {
            metadata,
            body: body.into(),
            url,
            output_path,
            source_path,
            depth,
            content_type: content_type.into(),
            is_list,
            list_items: None,
            taxonomies: HashMap::new(),
        })
    }

    #[must_use]
    pub fn relative_url(&self) -> &str {
        &self.url
    }

    pub fn add_taxonomy(&mut self, name: impl Into<String>, items: Vec<String>) {
        let _ = self.taxonomies.insert(name.into(), items);
    }

    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }
}
