use librawssg_config::ContentRule;
use librawssg_error as _;
use serde as _;
use serde_json::json;
use serde_yaml as _;
use tempfile as _;

macro_rules! must {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                eprintln!("{} failed: {}", $context, err);
                std::process::exit(1);
            }
        }
    };
}

#[test]
fn new_sets_required_fields() {
    let rule = ContentRule::new("blog", "**/*.md", "post");
    assert_eq!(rule.name, "blog");
    assert_eq!(rule.pattern, "**/*.md");
    assert_eq!(rule.template, "post");
    assert!(!rule.list_enabled);
    assert!(rule.list_template.is_none());
    assert!(rule.extra.is_empty());
}

#[test]
fn default_is_empty() {
    let rule = ContentRule::default();
    assert_eq!(rule.name, "");
    assert_eq!(rule.pattern, "");
    assert_eq!(rule.template, "");
    assert!(!rule.list_enabled);
    assert!(rule.list_template.is_none());
    assert!(rule.extra.is_empty());
}

#[test]
fn serialize_deserialize_roundtrip() {
    let mut rule = ContentRule::new("page", "**/*.html", "base");
    rule.list_enabled = true;
    rule.list_template = Some("list".into());
    let _ = rule.extra.insert("key".into(), json!("value"));
    let yaml = must!(serde_yaml::to_string(&rule), "serialize");
    let parsed: ContentRule = must!(serde_yaml::from_str(&yaml), "deserialize");
    assert_eq!(rule, parsed);
}
