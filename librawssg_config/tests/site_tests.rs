use librawssg_config::SiteConfig;
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
fn new_sets_site_name_only() {
    let site = SiteConfig::new("My Site");
    assert_eq!(site.site_name, "My Site");
    assert_eq!(site.language.as_deref(), Some("en"));
    assert!(site.navbar.is_empty());
    assert!(site.sidebar.is_empty());
}

#[test]
fn default_has_expected_values() {
    let site = SiteConfig::default();
    assert_eq!(site.site_name, "librawssg");
    assert_eq!(site.language.as_deref(), Some("en"));
    assert!(site.navbar.is_empty());
    assert!(site.sidebar.is_empty());
    assert!(site.description.is_none());
    assert!(site.base_url.is_none());
    assert!(site.author.is_none());
    assert!(site.repo_url.is_none());
    assert!(site.license.is_none());
    assert!(site.extra.is_empty());
}

#[test]
fn serialize_deserialize_roundtrip() {
    let mut site = SiteConfig::new("Test");
    let _ = site.extra.insert("foo".into(), json!("bar"));
    let yaml = must!(serde_yaml::to_string(&site), "serialize");
    let parsed: SiteConfig = must!(serde_yaml::from_str(&yaml), "deserialize");
    assert_eq!(site, parsed);
}
