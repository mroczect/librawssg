use librawssg_config::NavItem;
use librawssg_error as _;
use serde as _;
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
fn new_sets_label_and_url() {
    let item = NavItem::new("Home", "/");
    assert_eq!(item.label, "Home");
    assert_eq!(item.url, "/");
    assert!(item.children.is_empty());
}

#[test]
fn default_is_empty() {
    let item = NavItem::default();
    assert_eq!(item.label, "");
    assert_eq!(item.url, "");
    assert!(item.children.is_empty());
}

#[test]
fn children_can_be_added() {
    let mut parent = NavItem::new("Docs", "/docs");
    parent.children.push(NavItem::new("API", "/docs/api"));
    assert_eq!(parent.children.len(), 1);
    assert!(!parent.children.is_empty(), "child should exist");
    if let Some(child) = parent.children.first() {
        assert_eq!(child.label, "API");
    }
}

#[test]
fn serialize_deserialize_roundtrip() {
    let item = NavItem::new("Home", "/");
    let json = must!(serde_json::to_string(&item), "serialize");
    let parsed: NavItem = must!(serde_json::from_str(&json), "deserialize");
    assert_eq!(item, parsed);
}
