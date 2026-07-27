mod common;
use common::MockMarkdownRenderer;
use librawssg::error::RawssgError;
use librawssg::frontmatter::parse_frontmatter_and_render;
use std::path::Path;

#[test]
fn valid_frontmatter_parsed() {
    let input = "---\ntitle: Test\ndesc: Desc\n---\n# Content";
    let renderer = MockMarkdownRenderer::identity();
    let (fm, html) = parse_frontmatter_and_render(input, Path::new("file.md"), &renderer).unwrap();
    assert_eq!(fm.title, "Test");
    assert_eq!(fm.desc, "Desc");
    assert_eq!(html, "# Content");
}

#[test]
fn missing_opening_dashes_error() {
    let input = "# No frontmatter";
    let renderer = MockMarkdownRenderer::identity();
    let err = parse_frontmatter_and_render(input, Path::new("file.md"), &renderer).unwrap_err();
    match err {
        RawssgError::Frontmatter { path, source } => {
            assert!(source.to_string().contains("missing opening"));
            assert_eq!(path, Path::new("file.md"));
        }
        _ => panic!("wrong error"),
    }
}

#[test]
fn invalid_yaml_in_frontmatter() {
    let input = "---\ntitle: [broken\ndesc: x\n---\ncontent";
    let renderer = MockMarkdownRenderer::identity();
    assert!(parse_frontmatter_and_render(input, Path::new("f.md"), &renderer).is_err());
}

#[test]
fn missing_closing_dashes_error() {
    let input = "---\ntitle: Test\ndesc: Test desc\n";
    let renderer = MockMarkdownRenderer::identity();
    let err = parse_frontmatter_and_render(input, Path::new("f.md"), &renderer).unwrap_err();
    match err {
        RawssgError::Frontmatter { source, .. } => {
            assert!(source.to_string().contains("missing closing '---'"));
        }
        _ => panic!("expected Frontmatter error about missing closing dashes"),
    }
}

#[test]
fn windows_line_endings() {
    let input = "---\r\ntitle: Win\r\ndesc: Win desc\r\n---\r\n# Content";
    let renderer = MockMarkdownRenderer::identity();
    let (fm, html) = parse_frontmatter_and_render(input, Path::new("f.md"), &renderer).unwrap();
    assert_eq!(fm.title, "Win");
    assert_eq!(fm.desc, "Win desc");
    assert_eq!(html, "# Content");
}

#[test]
fn only_frontmatter_no_content() {
    let input = "---\ntitle: Only\ndesc: Only desc\n---";
    let renderer = MockMarkdownRenderer::identity();
    let (fm, html) = parse_frontmatter_and_render(input, Path::new("f.md"), &renderer).unwrap();
    assert_eq!(fm.title, "Only");
    assert_eq!(fm.desc, "Only desc");
    assert!(html.is_empty());
}

#[test]
fn empty_yaml_values_still_valid() {
    let input = "---\ntitle: \"\"\ndesc: \"\"\n---\n# Content after FM";
    let renderer = MockMarkdownRenderer::identity();
    let (fm, html) = parse_frontmatter_and_render(input, Path::new("f.md"), &renderer).unwrap();
    assert_eq!(fm.title, "");
    assert_eq!(fm.desc, "");
    assert_eq!(html, "# Content after FM");
}
