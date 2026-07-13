use super::ConfigLoader;
use crate::error::RawssgError;
use crate::types::GlobalConfig;
use std::path::Path;

pub struct YamlConfigLoader<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> YamlConfigLoader<P> {
    pub fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P: AsRef<Path>> ConfigLoader for YamlConfigLoader<P> {
    #[tracing::instrument(skip(self))]
    fn load(&self) -> Result<GlobalConfig, RawssgError> {
        let content = std::fs::read_to_string(self.path.as_ref())
            .map_err(|e| RawssgError::Config(format!("cannot read config file: {}", e)))?;
        serde_yaml::from_str(&content)
            .map_err(|e| RawssgError::Config(format!("invalid config YAML: {}", e)))
    }
}
