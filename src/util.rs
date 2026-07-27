use crate::error::RawssgError;
use crate::fs::FileSystem;
use std::path::{Component, Path, PathBuf};

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                if components
                    .last()
                    .is_some_and(|c: &Component| c != &Component::RootDir)
                {
                    components.pop();
                }
            }
            Component::CurDir => {}
            other => components.push(other),
        }
    }
    components.into_iter().collect()
}

pub fn safe_path(
    fs: &dyn FileSystem,
    base: &Path,
    candidate: &Path,
) -> Result<PathBuf, RawssgError> {
    let base_canon = fs.canonicalize(base).map_err(|e| {
        RawssgError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Cannot canonicalize base '{}': {}", base.display(), e),
        ))
    })?;

    let joined = if candidate.is_relative() {
        base_canon.join(candidate)
    } else {
        candidate.to_path_buf()
    };
    let normalized = normalize_path(&joined);

    if fs.exists(&normalized) {
        let resolved = fs.canonicalize(&normalized).map_err(|e| {
            RawssgError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Cannot resolve path '{}': {}", normalized.display(), e),
            ))
        })?;
        if !resolved.starts_with(&base_canon) {
            return Err(RawssgError::PathTraversal(format!(
                "Path traversal detected: {}",
                resolved.display()
            )));
        }
        Ok(resolved)
    } else {
        let parent = normalized.parent().ok_or_else(|| {
            RawssgError::PathTraversal(format!(
                "Cannot determine parent of output path: {}",
                normalized.display()
            ))
        })?;
        let parent_canon = fs.canonicalize(parent).map_err(|e| {
            RawssgError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Cannot resolve parent '{}': {}", parent.display(), e),
            ))
        })?;
        if !parent_canon.starts_with(&base_canon) {
            return Err(RawssgError::PathTraversal(format!(
                "Path escapes base: {}",
                normalized.display()
            )));
        }
        let file_name = normalized.file_name().ok_or_else(|| {
            RawssgError::PathTraversal(format!(
                "Output path has no file name: {}",
                normalized.display()
            ))
        })?;
        Ok(parent_canon.join(file_name))
    }
}
pub fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
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

pub fn match_pattern(pattern: &str, path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let segments: Vec<&str> = path_str.split('/').collect();
    let pattern_segments: Vec<&str> = pattern.split('/').collect();

    if pattern_segments.len() > segments.len() {
        return false;
    }

    for (i, pat) in pattern_segments.iter().enumerate() {
        if i >= segments.len() {
            return false;
        }
        if *pat == "**" {
            return true;
        }
        if !segment_matches(pat, segments[i]) {
            return false;
        }
    }

    pattern_segments.len() == segments.len()
}

fn segment_matches(pattern: &str, segment: &str) -> bool {
    let mut pattern_chars = pattern.chars();
    let mut segment_chars = segment.chars();

    loop {
        match pattern_chars.next() {
            Some('*') => {
                let rest_of_pattern: String = pattern_chars.clone().collect();
                if rest_of_pattern.is_empty() {
                    return true;
                }
                let mut remaining_segment: String = segment_chars.clone().collect();
                while !remaining_segment.is_empty() {
                    if segment_matches(&rest_of_pattern, &remaining_segment) {
                        return true;
                    }
                    segment_chars.next();
                    remaining_segment = segment_chars.clone().collect();
                }
                return false;
            }
            Some(pc) => match segment_chars.next() {
                Some(sc) if pc == sc => continue,
                _ => return false,
            },
            None => return segment_chars.next().is_none(),
        }
    }
}
