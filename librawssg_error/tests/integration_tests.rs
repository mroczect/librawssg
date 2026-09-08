use librawssg_error::{Error, Result};
use std::path::PathBuf;
use thiserror as _;

fn read_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

#[test]
fn io_error_propagates_via_question_mark() {
    let result = read_file("definitely_not_exists.txt");
    assert!(result.is_err());
    assert!(matches!(result, Err(Error::Io(_))));
}

#[test]
fn metadata_error_can_hold_boxed_dyn_error() {
    let source: Box<dyn core::error::Error + Send + Sync> =
        Box::new(std::io::Error::other("bad yaml"));
    let err = Error::Metadata {
        path: PathBuf::from("content/post.md"),
        source,
    };

    assert!(err.to_string().contains("content/post.md"));
    let as_core_error: &dyn core::error::Error = &err;
    let source_ref = as_core_error.source();
    assert!(source_ref.is_some(), "source should exist");
    if let Some(source_ref) = source_ref {
        assert_eq!(source_ref.to_string(), "bad yaml");
    }
}
