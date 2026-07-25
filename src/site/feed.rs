#[cfg(feature = "tera")]
use crate::error::RawssgError;
#[cfg(feature = "tera")]
use crate::site::TemplateRenderer;
#[cfg(feature = "tera")]
use crate::types::{PageContext, RawssgConfig};

#[cfg(feature = "tera")]
#[tracing::instrument(skip(renderer))]
pub fn generate_feed(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    posts: &[&PageContext],
    base_url: &str,
) -> Result<String, RawssgError> {
    let mut ctx = tera::Context::new();
    ctx.insert("site", &config.site);
    ctx.insert("posts", posts);
    ctx.insert("base_url", base_url);
    renderer.render(&config.generators.rss.template, &ctx)
}
