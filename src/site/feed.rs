use crate::error::RawssgError;
use crate::site::TemplateRenderer;
use crate::types::{PageContext, RawssgConfig};
use tera::Context;

#[tracing::instrument(skip(renderer))]
pub fn generate_feed(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    posts: &[&PageContext],
    base_url: &str,
) -> Result<String, RawssgError> {
    let mut ctx = Context::new();
    ctx.insert("site", &config.site);
    ctx.insert("posts", posts);
    ctx.insert("base_url", base_url);
    renderer.render(&config.generators.rss.template, &ctx)
}
