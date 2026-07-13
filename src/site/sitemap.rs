use crate::error::RawssgError;
use crate::site::TemplateRenderer;
use crate::types::PageContext;
use tera::Context;

#[tracing::instrument(skip(renderer))]
pub fn generate_sitemap(
    renderer: &dyn TemplateRenderer,
    pages: &[PageContext],
    base_url: &str,
) -> Result<String, RawssgError> {
    let mut ctx = Context::new();
    ctx.insert("pages", pages);
    ctx.insert("base_url", base_url);
    renderer.render("sitemap.xml", &ctx)
}
