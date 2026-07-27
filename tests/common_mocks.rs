#![allow(dead_code)]

#[cfg(feature = "tera")]
pub mod context_mocks {
    use librawssg::error::RawssgError;
    use librawssg::site::Context;
    use librawssg::site::context::{FeedContextBuilder, SitemapContextBuilder};
    use librawssg::types::{PageContext, RawssgConfig};

    pub struct MockFeedContextBuilder;

    impl FeedContextBuilder for MockFeedContextBuilder {
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

    pub struct MockSitemapContextBuilder;

    impl SitemapContextBuilder for MockSitemapContextBuilder {
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
}
