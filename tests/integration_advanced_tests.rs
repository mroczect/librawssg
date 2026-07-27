mod common;
mod common_mocks;

#[cfg(all(feature = "tera", feature = "pulldown"))]
mod advanced {
    use crate::common::MockFs;
    use crate::common_mocks::context_mocks::{MockFeedContextBuilder, MockSitemapContextBuilder};
    use librawssg::SiteBuilder;
    use librawssg::error::RawssgError;
    use librawssg::markdown::PulldownMarkdown;
    use librawssg::site::TeraRenderer;
    use librawssg::types::{ContentTypeDef, RawssgConfig};
    use std::path::PathBuf;

    #[test]
    fn builder_missing_markdown_renderer() {
        let mut config = RawssgConfig::default();
        config.content_types.push(ContentTypeDef {
            name: "page".into(),
            pattern: "*.md".into(),
            template: "base.html".into(),
            list_template: None,
            list_enabled: false,
        });
        let result = SiteBuilder::new()
            .config(config)
            .with_template_renderer(Box::new(TeraRenderer::new()))
            .build();
        match result {
            Err(RawssgError::Config(msg)) => assert!(msg.contains("markdown renderer")),
            _ => panic!("Expected config error about missing markdown renderer"),
        }
    }

    #[test]
    fn builder_missing_template_renderer() {
        let mut config = RawssgConfig::default();
        config.content_types.push(ContentTypeDef {
            name: "page".into(),
            pattern: "*.md".into(),
            template: "base.html".into(),
            list_template: None,
            list_enabled: false,
        });
        let result = SiteBuilder::new()
            .config(config)
            .with_markdown_renderer(Box::new(PulldownMarkdown))
            .build();
        match result {
            Err(RawssgError::Config(msg)) => assert!(msg.contains("template renderer")),
            _ => panic!("Expected config error about missing template renderer"),
        }
    }

    #[test]
    fn rss_enabled_without_context_builder_error() {
        let mut config = RawssgConfig::default();
        config.generators.rss.enabled = true;
        config.generators.rss.template = "rss.xml".into();
        config.generators.rss.path = "rss.xml".into();
        config.content_types.push(ContentTypeDef {
            name: "page".into(),
            pattern: "*.md".into(),
            template: "base.html".into(),
            list_template: None,
            list_enabled: false,
        });
        let result = SiteBuilder::new()
            .config(config)
            .with_template_renderer(Box::new(TeraRenderer::new()))
            .with_markdown_renderer(Box::new(PulldownMarkdown))
            .build();
        match result {
            Err(RawssgError::Config(msg)) => assert!(msg.contains("feed context builder")),
            _ => panic!("Expected error about missing feed context builder"),
        }
    }

    #[test]
    fn sitemap_enabled_without_context_builder_error() {
        let mut config = RawssgConfig::default();
        config.generators.sitemap.enabled = true;
        config.generators.sitemap.template = "sitemap.xml".into();
        config.generators.sitemap.path = "sitemap.xml".into();
        config.content_types.push(ContentTypeDef {
            name: "page".into(),
            pattern: "*.md".into(),
            template: "base.html".into(),
            list_template: None,
            list_enabled: false,
        });
        let result = SiteBuilder::new()
            .config(config)
            .with_template_renderer(Box::new(TeraRenderer::new()))
            .with_markdown_renderer(Box::new(PulldownMarkdown))
            .build();
        match result {
            Err(RawssgError::Config(msg)) => assert!(msg.contains("sitemap context builder")),
            _ => panic!("Expected error about missing sitemap context builder"),
        }
    }

    #[test]
    fn atomic_rename_fallback_on_cross_device() {
        let mut fs = MockFs::new();
        fs.add_file(
            "/tmp/content/about.md",
            "---\ntitle: About\ndesc: x\n---\nAbout",
        );
        fs.add_file(
            "/tmp/content/index.md",
            "---\ntitle: Index\ndesc: x\n---\nHome",
        );
        fs.dirs.push(PathBuf::from("/tmp/content"));
        fs.dirs.push(PathBuf::from("/tmp/dist"));
        fs.dirs.push(PathBuf::from("/tmp/dist.tmp"));

        let mut config = RawssgConfig::default();
        config.build.content_dir = "/tmp/content".into();
        config.build.output_dir = "/tmp/dist".into();
        config.build.templates_dir = "templates".into();
        config.content_types.push(ContentTypeDef {
            name: "page".into(),
            pattern: "**/*.md".into(),
            template: "base.html".into(),
            list_template: None,
            list_enabled: false,
        });

        let mock_renderer = crate::common::MockTemplateRenderer::new();
        let mock_md = crate::common::MockMarkdownRenderer::identity();

        let site = SiteBuilder::new()
            .config(config)
            .with_fs(Box::new(fs))
            .with_template_renderer(Box::new(mock_renderer))
            .with_markdown_renderer(Box::new(mock_md))
            .with_feed_context_builder(Box::new(MockFeedContextBuilder))
            .with_sitemap_context_builder(Box::new(MockSitemapContextBuilder))
            .build()
            .unwrap();

        let result = site.generate();
        assert!(result.is_ok());
    }
}
