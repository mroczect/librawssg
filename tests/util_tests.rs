use librawssg::error::RawssgError;
use librawssg::fs::real::RealFs;
use librawssg::util::{match_pattern, relative_prefix, safe_path, slugify};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn safe_path_inside_base() {
    let dir = tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let candidate = base.join("file.txt");
    std::fs::write(&candidate, "").unwrap();
    let result = safe_path(&RealFs, &base, &candidate).unwrap();
    assert_eq!(result, candidate.canonicalize().unwrap());
}

#[test]
fn safe_path_outside_base() {
    let dir = tempdir().unwrap();
    let base = dir.path().join("sub");
    std::fs::create_dir(&base).unwrap();
    let base = base.canonicalize().unwrap();

    // Create a file outside the base directory but inside the temp dir
    let outside = dir.path().join("outside.txt");
    std::fs::write(&outside, "test").unwrap();
    let outside = outside.canonicalize().unwrap(); // now works because the file exists

    match safe_path(&RealFs, &base, &outside) {
        Err(RawssgError::PathTraversal(_)) => {}
        _ => panic!("expected PathTraversal error"),
    }
}

#[test]
fn slugify_simple() {
    assert_eq!(slugify("Hello World"), "hello-world");
}

#[test]
fn slugify_special_chars() {
    assert_eq!(slugify("Rust & SSG!"), "rust-ssg");
}

#[test]
fn slugify_multiple_dashes() {
    assert_eq!(slugify("A--B"), "a-b");
}

#[test]
fn relative_prefix_depths() {
    assert_eq!(relative_prefix(0), "./");
    assert_eq!(relative_prefix(1), "../");
    assert_eq!(relative_prefix(3), "../../../");
}

#[test]
fn match_pattern_exact() {
    assert!(match_pattern("blog/post.md", Path::new("blog/post.md")));
}

#[test]
fn match_pattern_wildcard_single() {
    assert!(match_pattern("blog/*.md", Path::new("blog/hello.md")));
    assert!(!match_pattern("blog/*.md", Path::new("blog/sub/hello.md")));
}

#[test]
fn match_pattern_double_wildcard() {
    assert!(match_pattern("blog/**", Path::new("blog/a/b/c.md")));
    assert!(match_pattern("blog/**", Path::new("blog/file.md")));
}

#[test]
fn match_pattern_too_long() {
    assert!(!match_pattern("a/b/c", Path::new("a/b")));
}

#[test]
fn match_pattern_empty_pattern() {
    // Should not match non-empty path
    assert!(!match_pattern("", Path::new("anything")));
}
