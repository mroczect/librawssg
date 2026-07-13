use crate::error::RawssgError;
use crate::markdown::MarkdownRenderer;
use crate::types::PageFrontMatter;
use std::path::PathBuf;

#[tracing::instrument(skip(raw, renderer))]
pub fn parse_markdown(
    raw: &str,
    path: &PathBuf,
    renderer: &dyn MarkdownRenderer,
) -> Result<(PageFrontMatter, String), RawssgError> {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return Err(RawssgError::Frontmatter {
            path: path.clone(),
            source: "missing opening '---'".into(),
        });
    }

    let without_first = trimmed.trim_start_matches("---").trim_start();
    let end = without_first
        .find("\n---")
        .or_else(|| without_first.find("\r\n---"))
        .unwrap_or(without_first.len());

    let yaml_str = &without_first[..end];
    let markdown_str = without_first[end..]
        .trim_start_matches("\n---")
        .trim_start_matches("\r\n---")
        .trim();

    let fm: PageFrontMatter = serde_yaml::from_str(yaml_str).map_err(|e| {
        RawssgError::Frontmatter {
            path: path.clone(),
            source: Box::new(e),
        }
    })?;

    let html = renderer.render(markdown_str);
    Ok((fm, html))
}
