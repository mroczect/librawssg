pub mod builder;
pub mod feed;
pub mod page;
pub mod sitemap;

use crate::error::RawssgError;
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
