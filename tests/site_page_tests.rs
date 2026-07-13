mod common;
use common::{MockFs, MockMarkdownRenderer};
use librawssg::error::RawssgError;
use librawssg::site::page::build_page_context;
use std::path::Path;

#[test]
fn successful_page_context() {
    let mut fs = MockFs::new();
    fs.add_file(
        "/content/about.md",
        "---\ntitle: About\ndesc: About page\n---\nHello",
    );
    let renderer = MockMarkdownRenderer::identity();
    let ctx = build_page_context(
        &fs,
        &renderer,
        Path::new("/content/about.md"),
        Path::new("/content"),
    )
    .unwrap()
    .expect("should return some");
    assert_eq!(ctx.frontmatter.title, "About");
    assert_eq!(ctx.content_html, "Hello");
    assert_eq!(ctx.url, "about.html");
    assert_eq!(ctx.depth, 0);
    assert!(ctx.pub_date.is_none());
}

#[test]
fn draft_page_returns_none() {
    let mut fs = MockFs::new();
    fs.add_file(
        "/content/draft.md",
        "---\ntitle: Draft\ndesc: x\ndraft: true\n---\nContent",
    );
    let renderer = MockMarkdownRenderer::identity();
    let result = build_page_context(
        &fs,
        &renderer,
        Path::new("/content/draft.md"),
        Path::new("/content"),
    )
    .unwrap();
    assert!(result.is_none());
}

#[test]
fn date_formatting() {
    let mut fs = MockFs::new();
    fs.add_file(
        "/content/post.md",
        "---\ntitle: Post\ndesc: x\ndate: 2025-01-01\n---\nBody",
    );
    let renderer = MockMarkdownRenderer::identity();
    let ctx = build_page_context(
        &fs,
        &renderer,
        Path::new("/content/post.md"),
        Path::new("/content"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        ctx.pub_date,
        Some("Wed, 01 Jan 2025 00:00:00 +0000".to_string())
    );
}

#[test]
fn nested_depth_calculation() {
    let mut fs = MockFs::new();
    fs.add_file("/content/blog/post.md", "---\ntitle: Nested\ndesc: x\n---\n");
    let renderer = MockMarkdownRenderer::identity();
    let ctx = build_page_context(
        &fs,
        &renderer,
        Path::new("/content/blog/post.md"),
        Path::new("/content"),
    )
    .unwrap()
    .unwrap();
    assert_eq!(ctx.url, "blog/post.html");
    assert_eq!(ctx.depth, 1);
}

#[test]
fn read_error_propagates() {
    let mut fs = MockFs::new();
    fs.add_file("/content/fault.md", "anything");
    fs.read_error = Some(Path::new("/content/fault.md").to_path_buf());
    let renderer = MockMarkdownRenderer::identity();
    let err = build_page_context(
        &fs,
        &renderer,
        Path::new("/content/fault.md"),
        Path::new("/content"),
    )
    .unwrap_err();
    match err {
        RawssgError::Io(_) => {}
        _ => panic!("expected Io error"),
    }
}
