pub mod builders;
pub mod feed;
pub mod page;
pub mod sitemap;

use crate::error::RawssgError;
use crate::fs::FileSystem;
use crate::markdown::MarkdownRenderer;
use crate::types::PageContext;
use std::path::Path;

pub trait TemplateRenderer: Send + Sync {
    fn render(&self, template_name: &str, context: &dyn Context) -> Result<String, RawssgError>;
}

pub trait Context: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any;
}

#[cfg(feature = "tera")]
pub mod context;

#[cfg(feature = "tera")]
pub struct TeraRenderer {
    tera: tera::Tera,
}

#[cfg(feature = "tera")]
impl TeraRenderer {
    pub fn new() -> Self {
        Self {
            tera: tera::Tera::default(),
        }
    }

    pub fn add_raw_template(&mut self, name: &str, content: &str) -> Result<(), RawssgError> {
        self.tera
            .add_raw_template(name, content)
            .map_err(|e| RawssgError::Template(e.to_string()))
    }
}

#[cfg(feature = "tera")]
impl Default for TeraRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "tera")]
impl TemplateRenderer for TeraRenderer {
    fn render(&self, template: &str, context: &dyn Context) -> Result<String, RawssgError> {
        let ctx = context
            .as_any()
            .downcast_ref::<tera::Context>()
            .ok_or_else(|| RawssgError::Template("Invalid context type for Tera".into()))?;
        self.tera
            .render(template, ctx)
            .map_err(|e| RawssgError::Template(e.to_string()))
    }
}

#[cfg(feature = "tera")]
impl Context for tera::Context {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
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
        orig.extension().is_some_and(|e| e == "md")
    }

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
