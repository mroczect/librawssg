#[cfg(feature = "tera")]
use super::context::SitemapContextBuilder;
#[cfg(feature = "tera")]
use crate::error::RawssgError;
#[cfg(feature = "tera")]
use crate::site::TemplateRenderer;
#[cfg(feature = "tera")]
use crate::types::{PageContext, RawssgConfig};

#[cfg(feature = "tera")]
#[tracing::instrument(skip(renderer, context_builder))]
pub fn generate_sitemap(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    pages: &[PageContext],
    base_url: &str,
    context_builder: &dyn SitemapContextBuilder,
) -> Result<String, RawssgError> {
    let ctx = context_builder.build_sitemap_context(config, pages, base_url)?;
    renderer.render(&config.generators.sitemap.template, &*ctx)
}
