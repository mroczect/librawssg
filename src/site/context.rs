use crate::error::RawssgError;
use crate::site::Context;
use crate::types::{PageContext, RawssgConfig};

/// Builder untuk membuat konteks RSS feed
pub trait FeedContextBuilder: Send + Sync {
    fn build_feed_context(
        &self,
        config: &RawssgConfig,
        posts: &[&PageContext],
        base_url: &str,
    ) -> Result<Box<dyn Context>, RawssgError>;
}

/// Builder untuk membuat konteks sitemap
pub trait SitemapContextBuilder: Send + Sync {
    fn build_sitemap_context(
        &self,
        config: &RawssgConfig,
        pages: &[PageContext],
        base_url: &str,
    ) -> Result<Box<dyn Context>, RawssgError>;
}

#[cfg(feature = "tera")]
pub struct TeraFeedContextBuilder;

#[cfg(feature = "tera")]
impl FeedContextBuilder for TeraFeedContextBuilder {
    fn build_feed_context(
        &self,
        config: &RawssgConfig,
        posts: &[&PageContext],
        base_url: &str,
    ) -> Result<Box<dyn Context>, RawssgError> {
        let mut ctx = tera::Context::new();
        ctx.insert("site", &config.site);
        ctx.insert("posts", posts);
        ctx.insert("base_url", base_url);
        Ok(Box::new(ctx))
    }
}

#[cfg(feature = "tera")]
pub struct TeraSitemapContextBuilder;

#[cfg(feature = "tera")]
impl SitemapContextBuilder for TeraSitemapContextBuilder {
    fn build_sitemap_context(
        &self,
        config: &RawssgConfig,
        pages: &[PageContext],
        base_url: &str,
    ) -> Result<Box<dyn Context>, RawssgError> {
        let mut ctx = tera::Context::new();
        ctx.insert("site", &config.site);
        ctx.insert("pages", pages);
        ctx.insert("base_url", base_url);
        Ok(Box::new(ctx))
    }
}
