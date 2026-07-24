use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RawssgConfig {
    #[serde(default)]
    pub site: GlobalConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub content_types: Vec<ContentTypeDef>,
    #[serde(default)]
    pub generators: GeneratorsConfig,
}

impl RawssgConfig {
    pub fn validate(&self) -> Result<(), crate::error::RawssgError> {
        if self.site.site_name.trim().is_empty() {
            return Err(crate::error::RawssgError::Config(
                "site.name cannot be empty".into(),
            ));
        }

        for ct in &self.content_types {
            if ct.name.trim().is_empty() {
                return Err(crate::error::RawssgError::Config(format!(
                    "content_type name cannot be empty (pattern: {})",
                    ct.pattern
                )));
            }
            if ct.template.trim().is_empty() {
                return Err(crate::error::RawssgError::Config(format!(
                    "content_type '{}' must have a template",
                    ct.name
                )));
            }
            if let Err(e) = glob::Pattern::new(&ct.pattern) {
                return Err(crate::error::RawssgError::Config(format!(
                    "Invalid glob pattern '{}' for content_type '{}': {}",
                    ct.pattern, ct.name, e
                )));
            }
        }

        if self.generators.rss.enabled {
            if self.generators.rss.template.trim().is_empty() {
                return Err(crate::error::RawssgError::Config(
                    "rss.template is required when RSS enabled".into(),
                ));
            }
            if self.generators.rss.path.trim().is_empty() {
                return Err(crate::error::RawssgError::Config(
                    "rss.path is required when RSS enabled".into(),
                ));
            }
        }

        if self.generators.sitemap.enabled {
            if self.generators.sitemap.template.trim().is_empty() {
                return Err(crate::error::RawssgError::Config(
                    "sitemap.template is required when sitemap enabled".into(),
                ));
            }
            if self.generators.sitemap.path.trim().is_empty() {
                return Err(crate::error::RawssgError::Config(
                    "sitemap.path is required when sitemap enabled".into(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub navbar: Vec<NavItem>,
    #[serde(default)]
    pub sidebar: Vec<NavItem>,
    #[serde(default = "default_site_name")]
    pub site_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            navbar: vec![],
            sidebar: vec![],
            site_name: default_site_name(),
            description: None,
            language: Some("en".into()),
            base_url: None,
            author: None,
            repo_url: None,
            license: None,
        }
    }
}

fn default_site_name() -> String {
    "rawssg".into()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentTypeDef {
    pub name: String,
    pub pattern: String,
    pub template: String,
    #[serde(default)]
    pub list_template: Option<String>,
    #[serde(default)]
    pub list_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GeneratorsConfig {
    #[serde(default)]
    pub rss: GeneratorDef,
    #[serde(default)]
    pub sitemap: GeneratorDef,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GeneratorDef {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub template: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NavItem {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PageFrontMatter {
    pub title: String,
    pub desc: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub date: Option<NaiveDate>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub draft: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct PageContext {
    pub frontmatter: PageFrontMatter,
    pub content_html: String,
    pub url: String,
    pub file_path: String,
    pub depth: usize,
    pub pub_date: Option<String>,
    pub content_type: String,
    pub is_list: bool,
    pub list_items: Option<Vec<PageContext>>,
}
