use crate::error::RawssgError;
use std::path::{Path, PathBuf};

pub fn safe_path(base: &Path, candidate: &Path) -> Result<PathBuf, RawssgError> {
    let base = base.canonicalize().map_err(RawssgError::Io)?;
    let resolved = candidate.canonicalize().unwrap_or_else(|_| candidate.to_path_buf());
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
    if depth == 0 { "./".into() } else { "../".repeat(depth) }
}

pub fn match_pattern(pattern: &str, path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let segments: Vec<&str> = path_str.split('/').collect();
    let pattern_segments: Vec<&str> = pattern.split('/').collect();
    if pattern_segments.len() > segments.len() { return false; }
    for (i, pat) in pattern_segments.iter().enumerate() {
        if i >= segments.len() { return false; }
        if *pat == "**" { return true; }
        if *pat == "*" { continue; }
        if *pat != segments[i] { return false; }
    }
    pattern_segments.len() == segments.len()
}
