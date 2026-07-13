use librawssg::error::RawssgError;
use std::path::PathBuf;

#[test]
fn error_display_and_diagnostic_code() {
    let err = RawssgError::Frontmatter {
        path: PathBuf::from("test.md"),
        source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "oops")),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Failed to parse frontmatter in test.md"));
    // Diagnostic code
    assert_eq!(
        format!("{:?}", err),
        "Frontmatter { path: \"test.md\", source: Custom { kind: Other, error: \"oops\" } }"
    );
}
