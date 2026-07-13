pub mod builder;
pub mod feed;
pub mod page;
pub mod sitemap;

use crate::error::RawssgError;
use crate::fs::FileSystem;
use crate::markdown::MarkdownRenderer;
use crate::types::PageContext;
use std::path::Path;
use tera::Context;

pub trait TemplateRenderer: Send + Sync {
    fn render(&self, template: &str, context: &Context) -> Result<String, RawssgError>;
    fn add_raw_template(&mut self, name: &str, content: &str) -> Result<(), RawssgError>;
}

pub struct TeraRenderer {
    tera: tera::Tera,
}

impl TeraRenderer {
    pub fn new() -> Self {
        Self {
            tera: tera::Tera::default(),
        }
    }
}

impl TemplateRenderer for TeraRenderer {
    fn render(&self, template: &str, context: &Context) -> Result<String, RawssgError> {
        self.tera
            .render(template, context)
            .map_err(|e| RawssgError::Template(e.to_string()))
    }
    fn add_raw_template(&mut self, name: &str, content: &str) -> Result<(), RawssgError> {
        self.tera
            .add_raw_template(name, content)
            .map_err(|e| RawssgError::Template(e.to_string()))
    }
}

pub trait ContentHandler: Send + Sync {
    fn can_handle(&self, relative_path: &Path, original_path: &Path) -> bool;
    fn process(
        &self,
        fs: &dyn FileSystem,
        md_renderer: &dyn MarkdownRenderer,
        file_path: &Path,
        content_dir: &Path,
    ) -> Result<Option<PageContext>, RawssgError>;
}

pub struct MarkdownPageHandler;

impl ContentHandler for MarkdownPageHandler {
    fn can_handle(&self, _rel: &Path, orig: &Path) -> bool {
        orig.extension().map_or(false, |e| e == "md")
    }

    #[tracing::instrument(skip(self, fs, md_renderer))]
    fn process(
        &self,
        fs: &dyn FileSystem,
        md_renderer: &dyn MarkdownRenderer,
        file_path: &Path,
        content_dir: &Path,
    ) -> Result<Option<PageContext>, RawssgError> {
        crate::site::page::build_page_context(fs, md_renderer, file_path, content_dir)
    }
}

pub struct StaticFileHandler;

impl ContentHandler for StaticFileHandler {
    fn can_handle(&self, _rel: &Path, _orig: &Path) -> bool {
        true
    }

    fn process(
        &self,
        _fs: &dyn FileSystem,
        _md_renderer: &dyn MarkdownRenderer,
        _file_path: &Path,
        _content_dir: &Path,
    ) -> Result<Option<PageContext>, RawssgError> {
        Ok(None)
    }
}
