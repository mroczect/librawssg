mod common;
mod common_mocks;
#[cfg(feature = "tera")]
use crate::common_mocks::context_mocks::{MockFeedContextBuilder, MockSitemapContextBuilder};
use common::{MockMarkdownRenderer, MockTemplateRenderer};
use librawssg::site::builder::SiteBuilder;
use librawssg::types::{ContentTypeDef, RawssgConfig};
use std::fs;
use tempfile::tempdir;

fn make_config() -> RawssgConfig {
    let mut cfg = RawssgConfig::default();
    cfg.build.content_dir = "content".into();
    cfg.build.output_dir = "dist".into();
    cfg.build.templates_dir = "templates".into();
    cfg.build.static_dir = "static".into();
    cfg.content_types.push(ContentTypeDef {
        name: "blog".into(),
        pattern: "blog/*".into(),
        template: "post.html".into(),
        list_template: Some("blog_list.html".into()),
        list_enabled: true,
    });
    cfg
}

#[test]
fn builder_new_works() {
    let _builder = SiteBuilder::new();
}

#[test]
fn builder_load_config_file_not_found() {
    let result = SiteBuilder::new().load_config("nonexistent.yaml");
    assert!(result.is_err());
}

#[test]
#[cfg(feature = "tera")]
fn build_simple_site() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    let templates = dir.path().join("templates");
    let output = dir.path().join("dist");
    fs::create_dir(&content).unwrap();
    fs::create_dir(&templates).unwrap();

    fs::write(
        content.join("index.md"),
        "---\ntitle: Home\ndesc: Home page\n---\nHello",
    )
    .unwrap();
    fs::write(templates.join("base.html"), "{{ page_title }}").unwrap();

    let mut config = make_config();
    config.build.content_dir = content.to_string_lossy().into();
    config.build.output_dir = output.to_string_lossy().into();
    config.build.templates_dir = templates.to_string_lossy().into();

    let site = SiteBuilder::new()
        .config(config)
        .with_template_renderer(Box::new(MockTemplateRenderer::new()))
        .with_markdown_renderer(Box::new(MockMarkdownRenderer::identity()))
        .with_feed_context_builder(Box::new(MockFeedContextBuilder))
        .with_sitemap_context_builder(Box::new(MockSitemapContextBuilder))
        .build()
        .expect("build should succeed");

    assert_eq!(site.pages().len(), 1);
    assert_eq!(site.pages()[0].url, "index.html");
}

#[test]
#[cfg(feature = "tera")]
fn draft_skipped_during_build() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    let templates = dir.path().join("templates");
    fs::create_dir(&content).unwrap();
    fs::create_dir(&templates).unwrap();

    fs::write(
        content.join("draft.md"),
        "---\ntitle: Draft\ndesc: x\ndraft: true\n---\n...",
    )
    .unwrap();
    fs::write(templates.join("base.html"), "base").unwrap();

    let mut config = make_config();
    config.build.content_dir = content.to_string_lossy().into();
    config.build.output_dir = dir.path().join("dist").to_string_lossy().into();
    config.build.templates_dir = templates.to_string_lossy().into();

    let site = SiteBuilder::new()
        .config(config)
        .with_template_renderer(Box::new(MockTemplateRenderer::new()))
        .with_markdown_renderer(Box::new(MockMarkdownRenderer::identity()))
        .with_feed_context_builder(Box::new(MockFeedContextBuilder))
        .with_sitemap_context_builder(Box::new(MockSitemapContextBuilder))
        .build()
        .unwrap();
    assert!(site.pages().is_empty());
}

#[test]
#[cfg(feature = "tera")]
fn blog_list_page_generated() {
    let dir = tempdir().unwrap();
    let content = dir.path().join("content");
    let templates = dir.path().join("templates");
    fs::create_dir(&content).unwrap();
    fs::create_dir(&templates).unwrap();

    fs::create_dir(content.join("blog")).unwrap();
    fs::write(
        content.join("blog/post1.md"),
        "---\ntitle: P1\ndesc: x\ndate: 2025-01-01\n---\nOne",
    )
    .unwrap();
    fs::write(
        content.join("blog/post2.md"),
        "---\ntitle: P2\ndesc: x\ndate: 2025-01-02\n---\nTwo",
    )
    .unwrap();
    fs::write(templates.join("base.html"), "{{ page_title }}").unwrap();
    fs::write(templates.join("post.html"), "post").unwrap();
    fs::write(templates.join("blog_list.html"), "list").unwrap();

    let mut config = make_config();
    config.build.content_dir = content.to_string_lossy().into();
    config.build.output_dir = dir.path().join("dist").to_string_lossy().into();
    config.build.templates_dir = templates.to_string_lossy().into();

    let site = SiteBuilder::new()
        .config(config)
        .with_template_renderer(Box::new(MockTemplateRenderer::new()))
        .with_markdown_renderer(Box::new(MockMarkdownRenderer::identity()))
        .with_feed_context_builder(Box::new(MockFeedContextBuilder))
        .with_sitemap_context_builder(Box::new(MockSitemapContextBuilder))
        .build()
        .unwrap();

    assert_eq!(site.pages().len(), 3);
    let list = site.pages().iter().find(|p| p.is_list).unwrap();
    assert_eq!(list.url, "blog/index.html");
    assert_eq!(list.list_items.as_ref().unwrap().len(), 2);
}
