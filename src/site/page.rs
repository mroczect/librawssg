use crate::error::RawssgError;
use crate::frontmatter::parse_markdown;
use crate::fs::FileSystem;
use crate::markdown::MarkdownRenderer;
use crate::types::PageContext;
use crate::util::safe_path;
use chrono::{NaiveTime, TimeZone, Utc};
use std::path::Path;

#[tracing::instrument(skip(fs, markdown_renderer))]
pub fn build_page_context(
    fs: &dyn FileSystem,
    markdown_renderer: &dyn MarkdownRenderer,
    file_path: &Path,
    content_dir: &Path,
) -> Result<PageContext, RawssgError> {
    safe_path(content_dir, file_path)?;
    let raw = fs.read_to_string(file_path)?;
    let (fm, content_html) = parse_markdown(&raw, &file_path.to_path_buf(), markdown_renderer)?;

    if fm.draft {
        return Err(RawssgError::SiteGeneration(format!(
            "draft page skipped: {}",
            file_path.display()
        )));
    }

    let rel_path = file_path
        .strip_prefix(content_dir)
        .map_err(|e| RawssgError::SiteGeneration(e.to_string()))?;
    let url = rel_path.with_extension("html").to_string_lossy().to_string();
    let depth = rel_path.components().count().saturating_sub(1);

    let pub_date = fm.date.map(|d| {
        let dt = Utc
            .from_utc_datetime(&d.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()));
        dt.format("%a, %d %b %Y %H:%M:%S %z").to_string()
    });

    Ok(PageContext {
        frontmatter: fm,
        content_html,
        url,
        file_path: file_path.to_string_lossy().to_string(),
        depth,
        pub_date,
    })
}
