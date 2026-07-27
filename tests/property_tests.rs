use librawssg::util::{match_pattern, relative_prefix, slugify};
use proptest::prelude::*;

proptest! {
    #[test]
    fn slugify_does_not_panic(s in "\\PC*") {
        let _ = slugify(&s);
    }

    #[test]
    fn slugify_result_contains_no_uppercase(s in "\\PC*") {
        let result = slugify(&s);
        assert!(!result.chars().any(|c| c.is_uppercase()));
    }

    #[test]
    fn relative_prefix_never_empty_for_any_depth(depth in 0usize..100) {
        let prefix = relative_prefix(depth);
        assert!(!prefix.is_empty());
        assert!(prefix.ends_with('/'));
        assert_eq!(prefix.matches("../").count(), depth);
    }

    #[test]
    fn match_pattern_exact_self(path in "[a-zA-Z0-9_/]+\\.md") {
        let pattern = path.clone();
        assert!(match_pattern(&pattern, std::path::Path::new(&path)));
    }

    #[test]
    fn match_pattern_double_star_matches_everything(path in "[a-zA-Z0-9_/]+\\.md") {
        assert!(match_pattern("**", std::path::Path::new(&path)));
    }

    #[test]
    fn match_pattern_wildcard_segment(path in "blog/[a-z]+\\.md") {
        let filename = std::path::Path::new(&path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert!(match_pattern("blog/*.md", std::path::Path::new(&path)));
    }
}
