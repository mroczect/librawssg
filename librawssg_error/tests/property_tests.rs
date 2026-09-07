use librawssg_error::Error;
use thiserror as _;

#[test]
fn config_error_message_preserves_input() {
    let samples = [
        "",
        "short",
        "a very long error message with symbols !@#$%^&*()",
        "line1\nline2",
    ];

    for sample in samples {
        let err = Error::Config(sample.to_string());
        assert_eq!(err.to_string(), format!("Configuration error: {sample}"));
    }
}

#[test]
fn path_traversal_error_message_preserves_input() {
    let samples = [
        "../",
        "..\\..\\windows",
        "/absolute/path",
        "sub/../../escape",
    ];

    for sample in samples {
        let err = Error::PathTraversal(sample.to_string());
        assert_eq!(
            err.to_string(),
            format!("Path traversal attempt detected: {sample}")
        );
    }
}

#[test]
fn render_error_message_preserves_input() {
    let samples = [
        "missing variable `title`",
        "template not found: base.html",
        "unclosed tag",
    ];

    for sample in samples {
        let err = Error::Render(sample.to_string());
        assert_eq!(
            err.to_string(),
            format!("Template rendering error: {sample}")
        );
    }
}
