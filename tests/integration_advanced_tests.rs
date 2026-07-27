mod common;
mod common_mocks;

#[cfg(all(feature = "tera", feature = "pulldown"))]
mod advanced {
    use librawssg::SiteBuilder;
    use librawssg::error::RawssgError;
    use librawssg::markdown::PulldownMarkdown;
    use librawssg::site::TeraRenderer;
    use librawssg::types::{ContentTypeDef, RawssgConfig};

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
}
