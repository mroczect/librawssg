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

    fs::write(
        templates.join("base.html"),
        "<html><head><title>{{ page_title }}</title></head><body>{{ page_content }}</body></html>",
    )
    .unwrap();

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
    fs::write(templates.join("base.html"), "{{ page_content }}").unwrap();

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

    let site = SiteBuilder::new().config(config).build().unwrap();
    site.generate().unwrap();

    assert!(!output.join("draft.html").exists());
}
