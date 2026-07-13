use crate::error::RawssgError;
use std::path::{Path, PathBuf};
use crate::fs::FileSystem;

pub fn safe_path(
    fs: &dyn FileSystem,
    base: &Path,
    candidate: &Path,
) -> Result<PathBuf, RawssgError> {
    let base_canon = fs.canonicalize(base).map_err(RawssgError::Io)?;

    let full_candidate = if candidate.is_relative() {
        base_canon.join(candidate)
    } else {
        candidate.to_path_buf()
    };

    let resolved = fs
        .canonicalize(&full_candidate)
        .unwrap_or_else(|_| full_candidate.clone());

    if !resolved.starts_with(&base_canon) {
        return Err(RawssgError::PathTraversal(candidate.to_path_buf()));
    }

    Ok(resolved)
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
            // Matches zero or more directories
            return true;
        }
        if !segment_matches(pat, segments[i]) {
            return false;
        }
    }

    // After matching all pattern segments, the number of segments must be equal
    // unless the pattern ends with "**" (which we already returned true).
    pattern_segments.len() == segments.len()
}

/// Simple wildcard matching for a single path segment.
/// `*` matches any sequence of characters (except path separators, but segments don't contain them).
fn segment_matches(pattern: &str, segment: &str) -> bool {
    let mut pattern_chars = pattern.chars();
    let mut segment_chars = segment.chars();

    loop {
        match pattern_chars.next() {
            Some('*') => {
                // Greedily match as many characters as possible until the rest of the pattern matches.
                let rest_of_pattern: String = pattern_chars.clone().collect();
                if rest_of_pattern.is_empty() {
                    // * at the end matches everything left.
                    return true;
                }
                // Try to find the rest pattern in the remaining segment.
                // We'll consume segment chars until we find a match.
                let mut remaining_segment: String = segment_chars.clone().collect();
                while !remaining_segment.is_empty() {
                    if segment_matches(&rest_of_pattern, &remaining_segment) {
                        return true;
                    }
                    segment_chars.next(); // consume one more char from segment
                    remaining_segment = segment_chars.clone().collect();
                }
                return false;
            }
            Some(pc) => match segment_chars.next() {
                Some(sc) if pc == sc => continue,
                _ => return false,
            },
            None => return segment_chars.next().is_none(), // both must end
        }
    }
}
