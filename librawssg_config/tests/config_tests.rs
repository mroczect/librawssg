use librawssg_config::{Config, ContentRule};
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

fn valid_config() -> Config {
    let mut config = Config::new().with_site_name("My Site");
    config.add_content_rule(ContentRule::new("page", "**/*.html", "base"));
    config
}

#[test]
fn new_creates_default_with_empty_rules() {
    let config = Config::new();
    assert_eq!(config.site.site_name, "librawssg");
    assert!(config.content_rules.is_empty());
    assert!(config.extra.is_empty());
}

#[test]
fn with_site_name_updates_name() {
    let config = Config::new().with_site_name("Awesome");
    assert_eq!(config.site.site_name, "Awesome");
}

#[test]
fn add_and_find_rule() {
    let mut config = Config::new();
    config.add_content_rule(ContentRule::new("blog", "**/*.md", "post"));
    assert!(config.find_rule_by_name("blog").is_some());
    assert!(config.find_rule_by_name("missing").is_none());
}

#[test]
fn remove_rule_by_name_returns_removed() {
    let mut config = Config::new();
    config.add_content_rule(ContentRule::new("blog", "**/*.md", "post"));
    let removed = config.remove_rule_by_name("blog");
    assert!(removed.is_some());
    assert!(config.find_rule_by_name("blog").is_none());
}

#[test]
fn has_duplicate_rule_names_detects_duplicates() {
    let mut config = Config::new();
    config.add_content_rule(ContentRule::new("page", "**/*.html", "base"));
    config.add_content_rule(ContentRule::new("page", "**/*.raw", "raw"));
    assert!(config.has_duplicate_rule_names());
}

#[test]
fn valid_config_passes_validation() {
    let config = valid_config();
    assert!(config.validate().is_ok());
}

#[test]
fn empty_site_name_fails_validation() {
    let mut config = valid_config();
    config.site.site_name = "   ".to_string();
    let result = config.validate();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            matches!(err, librawssg_error::Error::Validation(ref msg) if msg == "site_name cannot be empty")
        );
    }
}

#[test]
fn no_content_rules_fails_validation() {
    let mut config = valid_config();
    config.content_rules.clear();
    let result = config.validate();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            matches!(err, librawssg_error::Error::Validation(ref msg) if msg == "at least one content rule must be defined")
        );
    }
}

#[test]
fn duplicate_rule_names_fails_validation() {
    let mut config = valid_config();
    config.add_content_rule(ContentRule::new("page", "**/*.raw", "raw"));
    let result = config.validate();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            matches!(err, librawssg_error::Error::Validation(ref msg) if msg == "duplicate content rule names are not allowed")
        );
    }
}

#[test]
fn content_rule_empty_name_fails_validation() {
    let mut config = valid_config();
    assert!(
        config.content_rules.first_mut().is_some(),
        "rule should exist"
    );
    if let Some(rule) = config.content_rules.first_mut() {
        rule.name = String::new();
    }
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn content_rule_empty_pattern_fails_validation() {
    let mut config = valid_config();
    assert!(
        config.content_rules.first_mut().is_some(),
        "rule should exist"
    );
    if let Some(rule) = config.content_rules.first_mut() {
        rule.pattern = String::new();
    }
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn content_rule_pattern_with_dotdot_fails_validation() {
    let mut config = valid_config();
    assert!(
        config.content_rules.first_mut().is_some(),
        "rule should exist"
    );
    if let Some(rule) = config.content_rules.first_mut() {
        rule.pattern = "**/../*.html".to_string();
    }
    let result = config.validate();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            matches!(err, librawssg_error::Error::Validation(ref msg) if msg.contains("contains invalid '..'"))
        );
    }
}

#[test]
fn content_rule_empty_template_fails_validation() {
    let mut config = valid_config();
    assert!(
        config.content_rules.first_mut().is_some(),
        "rule should exist"
    );
    if let Some(rule) = config.content_rules.first_mut() {
        rule.template = String::new();
    }
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn invalid_base_url_fails_validation() {
    let mut config = valid_config();
    config.site.base_url = Some("ftp://example.com".to_string());
    let result = config.validate();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            matches!(err, librawssg_error::Error::Validation(ref msg) if msg == "site.base_url must start with http:// or https://")
        );
    }
}

#[test]
fn valid_base_url_passes_validation() {
    let mut config = valid_config();
    config.site.base_url = Some("https://example.com".to_string());
    assert!(config.validate().is_ok());
}

#[test]
fn yaml_roundtrip() {
    let config = valid_config();
    let yaml = must!(config.to_yaml_string(), "to_yaml_string");
    let parsed = must!(Config::from_yaml_str(&yaml), "from_yaml_str");
    assert_eq!(config, parsed);
}

#[test]
fn json_roundtrip() {
    let config = valid_config();
    let json = must!(config.to_json_string(), "to_json_string");
    let parsed = must!(Config::from_json_str(&json), "from_json_str");
    assert_eq!(config, parsed);
}

#[test]
fn deserialize_from_yaml_with_extra_fields() {
    let yaml = r#"
site:
  site_name: Test Site
  description: A test
  extra:
    foo: bar
build:
  content_dir: src
content_rules:
  - name: page
    pattern: "**/*.raw"
    template: "main"
    list_enabled: true
"#;
    let config = must!(Config::from_yaml_str(yaml), "from_yaml_str");
    assert_eq!(config.site.site_name, "Test Site");
    assert_eq!(config.build.content_dir, "src");
    assert_eq!(config.content_rules.len(), 1);
    assert!(!config.content_rules.is_empty(), "rule should exist");
    if let Some(rule) = config.content_rules.first() {
        assert_eq!(rule.name, "page");
        assert!(rule.list_enabled);
    }
    let extra = config.site.extra.get("foo");
    assert!(extra.is_some());
    if let Some(value) = extra {
        assert_eq!(value, &json!("bar"));
    }
}

#[test]
fn deserialize_invalid_yaml_returns_config_error() {
    let result = Config::from_yaml_str("invalid_yaml: [");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, librawssg_error::Error::Config(_)));
    }
}

#[test]
fn deserialize_invalid_json_returns_config_error() {
    let result = Config::from_json_str("{invalid json");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, librawssg_error::Error::Config(_)));
    }
}
