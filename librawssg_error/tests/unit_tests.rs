use core::error::Error as CoreError;
use librawssg_error::Error;
use std::path::PathBuf;
use thiserror as _;

#[test]
fn display_for_config_error() {
    let err = Error::Config("invalid YAML".to_string());
    assert_eq!(err.to_string(), "Configuration error: invalid YAML");
}

#[test]
fn display_for_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let err = Error::Io(io_err);
    assert_eq!(err.to_string(), "I/O error: file missing");
}

#[test]
fn display_for_metadata_error() {
    let source = std::io::Error::new(std::io::ErrorKind::InvalidData, "bad front matter");
    let err = Error::Metadata {
        path: PathBuf::from("content/page.md"),
        source: Box::new(source),
    };
    assert_eq!(
        err.to_string(),
        "Failed to parse metadata in content/page.md"
    );
}

#[test]
fn display_for_render_error() {
    let err = Error::Render("template not found".to_string());
    assert_eq!(
        err.to_string(),
        "Template rendering error: template not found"
    );
}

#[test]
fn display_for_processor_error() {
    let err = Error::Processor("custom processor failed".to_string());
    assert_eq!(
        err.to_string(),
        "Content processor error: custom processor failed"
    );
}

#[test]
fn display_for_generator_error() {
    let err = Error::Generator("RSS generation failed".to_string());
    assert_eq!(err.to_string(), "Generator error: RSS generation failed");
}

#[test]
fn display_for_path_traversal_error() {
    let err = Error::PathTraversal("../escape".to_string());
    assert_eq!(
        err.to_string(),
        "Path traversal attempt detected: ../escape"
    );
}

#[test]
fn display_for_missing_config_error() {
    let err = Error::MissingConfig("base_url".to_string());
    assert_eq!(err.to_string(), "Missing configuration key: base_url");
}

#[test]
fn display_for_generation_error() {
    let err = Error::Generation("output write failed".to_string());
    assert_eq!(
        err.to_string(),
        "Site generation error: output write failed"
    );
}

#[test]
fn display_for_not_found_error() {
    let err = Error::NotFound("asset.css".to_string());
    assert_eq!(err.to_string(), "Resource not found: asset.css");
}

#[test]
fn display_for_serialization_error() {
    let err = Error::Serialization("invalid JSON".to_string());
    assert_eq!(err.to_string(), "Serialization error: invalid JSON");
}

#[test]
fn display_for_validation_error() {
    let err = Error::Validation("name too long".to_string());
    assert_eq!(err.to_string(), "Validation error: name too long");
}

#[test]
fn display_for_duplicate_error() {
    let err = Error::Duplicate("duplicate key".to_string());
    assert_eq!(err.to_string(), "Duplicate value: duplicate key");
}

#[test]
fn display_for_invalid_state_error() {
    let err = Error::InvalidState("unexpected null".to_string());
    assert_eq!(err.to_string(), "Invalid state: unexpected null");
}

#[test]
fn display_for_internal_error() {
    let err = Error::Internal("bug in code".to_string());
    assert_eq!(err.to_string(), "Internal error: bug in code");
}

#[test]
fn implements_std_error_trait() {
    let err = Error::Config("test".to_string());
    let as_core_error: &dyn CoreError = &err;
    assert!(as_core_error.source().is_none());
}

#[test]
fn metadata_source_is_accessible() {
    let source = std::io::Error::other("source err");
    let err = Error::Metadata {
        path: PathBuf::from("test.txt"),
        source: Box::new(source),
    };
    let as_core_error: &dyn CoreError = &err;
    let source_ref = as_core_error.source();
    assert!(source_ref.is_some(), "source should exist");
    if let Some(source_ref) = source_ref {
        assert_eq!(source_ref.to_string(), "source err");
    }
}

#[test]
fn io_error_from_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn result_alias_works() {
    let res: librawssg_error::Result<()> = Ok(());
    assert!(res.is_ok());
}

#[test]
fn debug_output_contains_variant_name() {
    let err = Error::Generator("boom".to_string());
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("Generator"));
}
