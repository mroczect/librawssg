use crate::Metadata;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub metadata: Metadata,
    pub body: String,
    pub url: String,
    pub source_path: PathBuf,
    pub depth: usize,
    pub content_type: String,
    pub is_list: bool,
    pub list_items: Option<Vec<Document>>,
}
