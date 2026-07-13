use crate::error::RawssgError;
use std::path::{Path, PathBuf};

pub fn safe_path(base: &Path, candidate: &Path) -> Result<PathBuf, RawssgError> {
    let base = base
        .canonicalize()
        .map_err(RawssgError::Io)?;
    let resolved = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.to_path_buf());
    if !resolved.starts_with(&base) {
        return Err(RawssgError::PathTraversal(candidate.to_path_buf()));
    }
    Ok(resolved)
}

pub fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn relative_prefix(depth: usize) -> String {
    if depth == 0 {
        "./".into()
    } else {
        "../".repeat(depth)
    }
}
