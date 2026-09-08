use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BuildConfig {
    #[serde(default = "default_content_dir")]
    pub content_dir: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_templates_dir")]
    pub templates_dir: String,
    #[serde(default = "default_static_dir")]
    pub static_dir: String,
}

fn default_content_dir() -> String {
    "content".into()
}
fn default_output_dir() -> String {
    "dist".into()
}
fn default_templates_dir() -> String {
    "templates".into()
}
fn default_static_dir() -> String {
    "static".into()
}

impl BuildConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            content_dir: default_content_dir(),
            output_dir: default_output_dir(),
            templates_dir: default_templates_dir(),
            static_dir: default_static_dir(),
        }
    }
}
