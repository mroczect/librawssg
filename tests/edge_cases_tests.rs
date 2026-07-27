use librawssg::config::{ConfigLoader, loader::YamlConfigLoader};
use librawssg::error::RawssgError;
use librawssg::types::RawssgConfig;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn load_or_default_uses_default_on_error() {
    struct BadLoader;
    impl ConfigLoader for BadLoader {
        fn load(&self) -> Result<RawssgConfig, RawssgError> {
            Err(RawssgError::Internal("boom".into()))
        }
    }
    let config = BadLoader.load_or_default();
    assert_eq!(config.site.site_name, "rawssg");
}

#[test]
fn load_or_default_returns_loaded_config_when_ok() {
    struct GoodLoader;
    impl ConfigLoader for GoodLoader {
        fn load(&self) -> Result<RawssgConfig, RawssgError> {
            let mut cfg = RawssgConfig::default();
            cfg.site.site_name = "custom".into();
            Ok(cfg)
        }
    }
    let config = GoodLoader.load_or_default();
    assert_eq!(config.site.site_name, "custom");
}

#[test]
fn yaml_loader_invalid_yaml_error() {
    let yaml = "site:\n  site_name: [invalid";
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    let loader = YamlConfigLoader::new(file.path());
    let err = loader.load().unwrap_err();
    match err {
        RawssgError::Config(msg) => assert!(msg.contains("invalid config YAML")),
        _ => panic!("Expected Config error"),
    }
}

#[test]
fn yaml_loader_file_not_found_error() {
    let loader = YamlConfigLoader::new("/nonexistent/path.yaml");
    let err = loader.load().unwrap_err();
    match err {
        RawssgError::Config(msg) => assert!(msg.contains("cannot read config")),
        _ => panic!("Expected Config error"),
    }
}
