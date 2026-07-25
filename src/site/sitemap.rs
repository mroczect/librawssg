#[cfg(feature = "tera")]
use crate::error::RawssgError;
#[cfg(feature = "tera")]
use crate::site::TemplateRenderer;
#[cfg(feature = "tera")]
use crate::types::PageContext;

#[cfg(feature = "tera")]
#[tracing::instrument(skip(renderer))]
pub fn generate_sitemap(
    renderer: &dyn TemplateRenderer,
    pages: &[PageContext],
    base_url: &str,
) -> Result<String, RawssgError> {
    let mut ctx = tera::Context::new();
    ctx.insert("pages", pages);
    ctx.insert("base_url", base_url);
    renderer.render("sitemap.xml", &ctx)
}
