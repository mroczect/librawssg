use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum RawssgError {
    #[error("I/O error")]
    #[diagnostic(code(rawssg::io))]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    #[diagnostic(code(rawssg::config))]
    Config(String),

    #[error("Failed to parse frontmatter in {path}")]
    #[diagnostic(
        code(rawssg::frontmatter),
        help("Check the YAML frontmatter syntax and ensure the file starts with '---'")
    )]
    Frontmatter {
        path: PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Template rendering error: {0}")]
    #[diagnostic(code(rawssg::template))]
    Template(String),

    #[error("Path traversal attempt detected: {0}")]
    #[diagnostic(
        code(rawssg::path_traversal),
        help("All file paths must stay within the project directory")
    )]
    PathTraversal(PathBuf),

    #[error("Missing configuration key: {0}")]
    #[diagnostic(code(rawssg::missing_config))]
    MissingConfig(String),

    #[error("Markdown processing error: {0}")]
    #[diagnostic(code(rawssg::markdown))]
    Markdown(String),

    #[error("Site generation error: {0}")]
    #[diagnostic(code(rawssg::site))]
    SiteGeneration(String),

    #[error("Resource not found: {0}")]
    #[diagnostic(code(rawssg::not_found))]
    NotFound(String),

    #[error("Internal error: {0}")]
    #[diagnostic(code(rawssg::internal))]
    Internal(String),
}
