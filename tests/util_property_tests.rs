use librawssg::util::slugify;
use proptest::prelude::*;

proptest! {
    #[test]
    fn slugify_no_consecutive_dashes(s in "\\PC*") {
        let result = slugify(&s);
        assert!(!result.contains("--"));
    }

    #[test]
    fn slugify_no_leading_or_trailing_dash(s in "\\PC*") {
        let result = slugify(&s);
        if !result.is_empty() {
            assert!(!result.starts_with('-'));
            assert!(!result.ends_with('-'));
        }
    }
}
