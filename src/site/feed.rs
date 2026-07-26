#[cfg(feature = "tera")]
use super::context::FeedContextBuilder;
#[cfg(feature = "tera")]
use crate::error::RawssgError;
#[cfg(feature = "tera")]
use crate::site::TemplateRenderer;
#[cfg(feature = "tera")]
use crate::types::{PageContext, RawssgConfig};

#[cfg(feature = "tera")]
#[tracing::instrument(skip(renderer, context_builder))]
pub fn generate_feed(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    posts: &[&PageContext],
    base_url: &str,
    context_builder: &dyn FeedContextBuilder,
) -> Result<String, RawssgError> {
    let ctx = context_builder.build_feed_context(config, posts, base_url)?;
    renderer.render(&config.generators.rss.template, &*ctx)
}
