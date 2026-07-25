#[cfg(all(feature = "tera", feature = "pulldown"))]
mod integration_tests {
    use librawssg::markdown::PulldownMarkdown;
    use librawssg::site::TeraRenderer;
    use librawssg::site::builder::SiteBuilder;
    use librawssg::types::{ContentTypeDef, RawssgConfig};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn full_site_generation() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        let templates = dir.path().join("templates");
        let output = dir.path().join("dist");
        fs::create_dir(&content).unwrap();
        fs::create_dir(&templates).unwrap();

        fs::write(
            content.join("index.md"),
            "---\ntitle: Home\ndesc: Home page\n---\nWelcome!",
        )
        .unwrap();
        fs::write(
            content.join("about.md"),
            "---\ntitle: About\ndesc: About us\n---\nAbout us",
        )
        .unwrap();

        let mut tera = TeraRenderer::new();
        tera.add_raw_template(
            "base.html",
            "<html><head><title>{{ page_title }}</title></head><body>{{ page_content }}</body></html>",
        ).unwrap();
        let md = PulldownMarkdown;

        let mut config = RawssgConfig::default();
        config.build.content_dir = content.to_string_lossy().into();
        config.build.output_dir = output.to_string_lossy().into();
        config.build.templates_dir = templates.to_string_lossy().into();

        config.content_types.push(ContentTypeDef {
            name: "page".into(),
            pattern: "**/*.md".into(),
            template: "base.html".into(),
            list_template: None,
            list_enabled: false,
        });

        let site = SiteBuilder::new()
            .config(config)
            .with_template_renderer(Box::new(tera))
            .with_markdown_renderer(Box::new(md))
            .build()
            .expect("build failed");
        site.generate().expect("generate failed");

        assert!(output.join("index.html").exists());
        assert!(output.join("about.html").exists());

        let home_html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(home_html.contains("Welcome!"));
        assert!(home_html.contains("<title>Home</title>"));
    }

    #[test]
    fn draft_not_in_output() {
        let dir = tempdir().unwrap();
        let content = dir.path().join("content");
        let templates = dir.path().join("templates");
        let output = dir.path().join("dist");
        fs::create_dir(&content).unwrap();
        fs::create_dir(&templates).unwrap();

        fs::write(
            content.join("draft.md"),
            "---\ntitle: Secret\ndesc: x\ndraft: true\n---\nShh",
        )
        .unwrap();

        let mut tera = TeraRenderer::new();
        tera.add_raw_template("base.html", "{{ page_content }}")
            .unwrap();
        let md = PulldownMarkdown;

        let mut config = RawssgConfig::default();
        config.build.content_dir = content.to_string_lossy().into();
        config.build.output_dir = output.to_string_lossy().into();
        config.build.templates_dir = templates.to_string_lossy().into();

        config.content_types.push(ContentTypeDef {
            name: "page".into(),
            pattern: "**/*.md".into(),
            template: "base.html".into(),
            list_template: None,
            list_enabled: false,
        });

        let site = SiteBuilder::new()
            .config(config)
            .with_template_renderer(Box::new(tera))
            .with_markdown_renderer(Box::new(md))
            .build()
            .unwrap();
        site.generate().unwrap();

        assert!(!output.join("draft.html").exists());
    }
}
