use std::path::Path;

#[must_use]
pub fn match_pattern(pattern: &str, path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let segments: Vec<&str> = path_str.split('/').collect();
    let pattern_segments: Vec<&str> = pattern.split('/').collect();
    match_pattern_slice(&pattern_segments, &segments)
}

fn match_pattern_slice(pattern: &[&str], segments: &[&str]) -> bool {
    match (pattern.first(), segments.first()) {
        (None, None) => true,
        (Some(_), None) => pattern.iter().all(|&p| p == "**"),
        (None, Some(_)) => false,
        (Some(&first_pat), Some(&first_seg)) => {
            if first_pat == "**" {
                if pattern.len() == 1 {
                    return true;
                }
                let Some(rest_pattern) = pattern.get(1..) else {
                    return false;
                };
                for i in 0..segments.len() {
                    let Some(rest_segments) = segments.get(i..) else {
                        continue;
                    };
                    if match_pattern_slice(rest_pattern, rest_segments) {
                        return true;
                    }
                }
                false
            } else if segment_matches(first_pat, first_seg) {
                match (pattern.get(1..), segments.get(1..)) {
                    (Some(next_pattern), Some(next_segments)) => {
                        match_pattern_slice(next_pattern, next_segments)
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
    }
}

fn segment_matches(pattern: &str, segment: &str) -> bool {
    let mut pattern_iter = pattern.chars();
    let mut segment_iter = segment.chars();

    loop {
        match pattern_iter.next() {
            Some('*') => {
                let rest: String = pattern_iter.clone().collect();
                if rest.is_empty() {
                    return true;
                }
                let mut remaining: String = segment_iter.clone().collect();
                while !remaining.is_empty() {
                    if segment_matches(&rest, &remaining) {
                        return true;
                    }
                    if segment_iter.next().is_none() {
                        break;
                    }
                    remaining = segment_iter.clone().collect();
                }
                return false;
            }
            Some(pc) => match segment_iter.next() {
                Some(sc) if pc == sc => {}
                _ => return false,
            },
            None => return segment_iter.next().is_none(),
        }
    }
}
