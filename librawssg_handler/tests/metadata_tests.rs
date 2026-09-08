use librawssg_fs as _;
use librawssg_handler::Metadata;
use serde as _;
use serde_json::json;

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
fn new_with_valid_title_ok() {
    let meta = must!(Metadata::new("My Title", "My Description"), "Metadata::new");
    assert_eq!(meta.title, "My Title");
    assert_eq!(meta.description, "My Description");
    assert!(!meta.draft);
    assert!(meta.tags.is_empty());
    assert!(meta.extra.is_empty());
}

#[test]
fn new_with_empty_title_errors() {
    let result = Metadata::new("", "Desc");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            librawssg_error::Error::Validation(ref msg) if msg == "metadata title cannot be empty"
        ));
    }
}

#[test]
fn new_with_whitespace_title_errors() {
    let result = Metadata::new("   ", "Desc");
    assert!(result.is_err());
}

#[test]
fn is_draft_reflects_field() {
    let mut meta = must!(Metadata::new("Title", "Desc"), "Metadata::new");
    assert!(!meta.is_draft());
    meta.draft = true;
    assert!(meta.is_draft());
}

#[test]
fn insert_extra_and_get() {
    let mut meta = must!(Metadata::new("Title", "Desc"), "Metadata::new");
    meta.insert_extra("key", "value");
    let val = meta.get_extra("key");
    assert!(val.is_some());
    if let Some(v) = val {
        assert_eq!(v, &json!("value"));
    }
}

#[test]
fn insert_extra_overwrites_existing() {
    let mut meta = must!(Metadata::new("Title", "Desc"), "Metadata::new");
    meta.insert_extra("key", "first");
    meta.insert_extra("key", "second");
    let val = meta.get_extra("key");
    assert!(val.is_some());
    if let Some(v) = val {
        assert_eq!(v, &json!("second"));
    }
}

#[test]
fn get_extra_missing_returns_none() {
    let meta = must!(Metadata::new("Title", "Desc"), "Metadata::new");
    assert!(meta.get_extra("nonexistent").is_none());
}

#[test]
fn default_metadata_has_empty_fields() {
    let meta = Metadata::default();
    assert_eq!(meta.title, "");
    assert_eq!(meta.description, "");
    assert!(!meta.draft);
    assert!(meta.tags.is_empty());
    assert!(meta.extra.is_empty());
    assert!(meta.author.is_none());
    assert!(meta.date.is_none());
    assert!(meta.updated.is_none());
}

#[test]
fn serialization_roundtrip() {
    let meta = must!(Metadata::new("Title", "Desc"), "Metadata::new");
    let json_str = must!(serde_json::to_string(&meta), "serialize");
    let deserialized: Metadata = must!(serde_json::from_str(&json_str), "deserialize");
    assert_eq!(meta, deserialized);
}

#[test]
fn deserialize_from_json_with_extra() {
    let json_str = r#"{
        "title": "Hello",
        "description": "World",
        "author": "Alice",
        "date": "2026-09-08",
        "tags": ["rust", "ssg"],
        "draft": false,
        "extra": {"foo": "bar"}
    }"#;
    let meta: Metadata = must!(serde_json::from_str(json_str), "deserialize");
    assert_eq!(meta.title, "Hello");
    assert_eq!(meta.description, "World");
    assert_eq!(meta.author.as_deref(), Some("Alice"));
    let expected_date = chrono::NaiveDate::from_ymd_opt(2026, 9, 8);
    assert_eq!(meta.date, expected_date);
    assert_eq!(meta.tags, vec!["rust", "ssg"]);
    assert!(!meta.draft);
    let extra_val = meta.get_extra("foo");
    assert!(extra_val.is_some());
    if let Some(v) = extra_val {
        assert_eq!(v, &json!("bar"));
    }
}
