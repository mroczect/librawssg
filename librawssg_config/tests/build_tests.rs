use librawssg_config::BuildConfig;
use librawssg_error as _;
use serde as _;
use serde_json as _;
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
fn default_values_are_correct() {
    let build = BuildConfig::default();
    assert_eq!(build.content_dir, "content");
    assert_eq!(build.output_dir, "dist");
    assert_eq!(build.templates_dir, "templates");
    assert_eq!(build.static_dir, "static");
}

#[test]
fn new_equals_default() {
    let build = BuildConfig::new();
    assert_eq!(build, BuildConfig::default());
}

#[test]
fn serialize_deserialize_roundtrip_yaml() {
    let build = BuildConfig::default();
    let yaml = must!(serde_yaml::to_string(&build), "serialize yaml");
    let parsed: BuildConfig = must!(serde_yaml::from_str(&yaml), "deserialize yaml");
    assert_eq!(build, parsed);
}

#[test]
fn deserialize_from_yaml_with_defaults() {
    let yaml = "content_dir: custom_content\noutput_dir: public\n";
    let build: BuildConfig = must!(serde_yaml::from_str(yaml), "deserialize yaml");
    assert_eq!(build.content_dir, "custom_content");
    assert_eq!(build.output_dir, "public");
    assert_eq!(build.templates_dir, "templates");
    assert_eq!(build.static_dir, "static");
}
