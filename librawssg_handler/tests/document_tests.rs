use chrono as _;
use librawssg_fs as _;
use librawssg_handler::{Document, Metadata};
use serde as _;
use serde_json as _;
use std::path::PathBuf;

macro_rules! must {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                eprintln!("{} failed: {}", $context, err);
                std::process::exit(1);
            }
        }
    };
}

fn valid_metadata() -> Metadata {
    must!(Metadata::new("Title", "Description"), "Metadata::new")
}

fn valid_document() -> Document {
    must!(
        Document::new(
            valid_metadata(),
            "<p>Body</p>",
            "blog/my-post.html",
            "blog/my-post/index.html",
            "content/blog/my-post.md",
            1,
            "blog",
            false,
        ),
        "Document::new"
    )
}

#[test]
fn new_valid_document_ok() {
    let doc = valid_document();
    assert_eq!(doc.metadata.title, "Title");
    assert_eq!(doc.body, "<p>Body</p>");
    assert_eq!(doc.url, "blog/my-post.html");
    assert_eq!(doc.output_path, PathBuf::from("blog/my-post/index.html"));
    assert_eq!(doc.source_path, PathBuf::from("content/blog/my-post.md"));
    assert_eq!(doc.depth, 1);
    assert_eq!(doc.content_type, "blog");
    assert!(!doc.is_list);
    assert!(doc.list_items.is_none());
    assert!(doc.taxonomies.is_empty());
}

#[test]
fn new_empty_url_errors() {
    let result = Document::new(
        valid_metadata(),
        "body",
        "",
        "out",
        "src.md",
        0,
        "page",
        false,
    );
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            librawssg_error::Error::Validation(ref msg) if msg == "document url cannot be empty"
        ));
    }
}

#[test]
fn new_empty_output_path_errors() {
    let result = Document::new(
        valid_metadata(),
        "body",
        "url",
        "",
        "src.md",
        0,
        "page",
        false,
    );
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            librawssg_error::Error::Validation(ref msg) if msg == "document output_path cannot be empty"
        ));
    }
}

#[test]
fn new_source_path_without_filename_errors() {
    let result = Document::new(valid_metadata(), "body", "url", "out", "", 0, "page", false);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            librawssg_error::Error::Validation(ref msg) if msg == "document source_path must have a file name"
        ));
    }
}

#[test]
fn new_depth_too_large_errors() {
    let result = Document::new(
        valid_metadata(),
        "body",
        "url",
        "out",
        "src.md",
        1001,
        "page",
        false,
    );
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            librawssg_error::Error::Validation(ref msg) if msg == "document depth is unreasonably large"
        ));
    }
}

#[test]
fn relative_url_returns_url() {
    let doc = valid_document();
    assert_eq!(doc.relative_url(), "blog/my-post.html");
}

#[test]
fn add_taxonomy_inserts() {
    let mut doc = valid_document();
    doc.add_taxonomy("categories", vec!["rust".to_string(), "ssg".to_string()]);
    let tax = doc.taxonomies.get("categories");
    assert!(tax.is_some());
    if let Some(items) = tax {
        assert_eq!(items, &vec!["rust".to_string(), "ssg".to_string()]);
    }
}

#[test]
fn depth_returns_depth() {
    let doc = valid_document();
    assert_eq!(doc.depth(), 1);
}

#[test]
fn list_items_default_none() {
    let doc = valid_document();
    assert!(doc.list_items.is_none());
}

#[test]
fn taxonomies_default_empty() {
    let doc = valid_document();
    assert!(doc.taxonomies.is_empty());
}
