use crate::error::RawssgError;
use crate::site::TemplateRenderer;
use crate::types::{GlobalConfig, PageContext};
use tera::Context;

#[tracing::instrument(skip(renderer))]
pub fn generate_feed(
    renderer: &dyn TemplateRenderer,
    config: &GlobalConfig,
    posts: &[PageContext],
    base_url: &str,
) -> Result<String, RawssgError> {
    let mut ctx = Context::new();
    ctx.insert("config", config);
    ctx.insert("posts", posts);
    ctx.insert("base_url", base_url);
    renderer.render("rss.xml", &ctx)
}
