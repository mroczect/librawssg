use librawssg::config::{ConfigLoader, loader::YamlConfigLoader};
use librawssg::error::RawssgError;
use librawssg::types::RawssgConfig;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn default_config_returns_valid_default() {
    let loader = librawssg::config::loader::DefaultConfig;
    let config = loader.load().expect("default config should load");
    assert_eq!(config.site.site_name, "rawssg");
    assert_eq!(config.build.content_dir, "content");
    assert!(
        !config.content_types.is_empty(),
        "default config must have at least one content type"
    );
    assert_eq!(config.content_types.len(), 1);
    assert_eq!(config.content_types[0].name, "page");
    assert_eq!(config.content_types[0].pattern, "**/*.md");
    assert_eq!(config.content_types[0].template, "base.html");
}

#[test]
fn yaml_loader_loads_full_config() {
    let yaml = r#"
site:
  site_name: "Test Site"
  description: "A test"
build:
  content_dir: "my_content"
  output_dir: "public"
content_types:
  - name: blog
    pattern: "blog/*.md"
    template: "post.html"
"#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    let loader = YamlConfigLoader::new(file.path());
    let config = loader.load().unwrap();
    assert_eq!(config.site.site_name, "Test Site");
    assert_eq!(config.build.content_dir, "my_content");
    assert_eq!(config.content_types.len(), 1);
    assert_eq!(config.content_types[0].name, "blog");
}

#[test]
fn yaml_loader_partial_defaults() {
    let yaml = "site:\n  site_name: \"OnlyName\"";
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    let loader = YamlConfigLoader::new(file.path());
    let config = loader.load().unwrap();
    assert_eq!(config.site.site_name, "OnlyName");
    assert_eq!(config.build.content_dir, "content");
}

#[test]
fn yaml_loader_invalid_syntax_error() {
    let yaml = "site: [unclosed";
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    let loader = YamlConfigLoader::new(file.path());
    match loader.load() {
        Err(RawssgError::Config(_)) => {}
        _ => panic!("expected Config error"),
    }
}

#[test]
fn yaml_loader_file_not_found() {
    let loader = YamlConfigLoader::new("nonexistent.yaml");
    match loader.load() {
        Err(RawssgError::Config(_)) => {}
        _ => panic!("expected Config error"),
    }
}

#[test]
fn config_loader_trait_load_or_default_on_error() {
    struct FailingLoader;
    impl ConfigLoader for FailingLoader {
        fn load(&self) -> Result<RawssgConfig, RawssgError> {
            Err(RawssgError::Config("fail".into()))
        }
    }
    let config = FailingLoader.load_or_default();
    assert_eq!(config.site.site_name, "rawssg");
}
