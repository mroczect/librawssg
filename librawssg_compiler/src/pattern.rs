use std::path::Path;

#[must_use]
pub fn match_pattern(pattern: &str, path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let segments: Vec<&str> = path_str.split('/').collect();
    let pattern_segments: Vec<&str> = pattern.split('/').collect();
    match_pattern_slice(&pattern_segments, &segments)
}

fn match_pattern_slice(pattern: &[&str], segments: &[&str]) -> bool {
    if pattern.is_empty() {
        return segments.is_empty();
    }
    if segments.is_empty() {
        return pattern.iter().all(|&p| p == "**");
    }

    match pattern[0] {
        "**" => {
            if pattern.len() == 1 {
                return true;
            }
            for i in 0..segments.len() {
                if match_pattern_slice(&pattern[1..], &segments[i..]) {
                    return true;
                }
            }
            false
        }
        pat => {
            if !segment_matches(pat, segments[0]) {
                return false;
            }
            match_pattern_slice(&pattern[1..], &segments[1..])
        }
    }
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
