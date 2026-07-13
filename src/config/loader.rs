use super::ConfigLoader;
use crate::error::RawssgError;
use crate::types::RawssgConfig;
use std::path::Path;

pub struct YamlConfigLoader<P: AsRef<Path> + Send + Sync> {
    path: P,
}

impl<P: AsRef<Path> + Send + Sync> YamlConfigLoader<P> {
    pub fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P: AsRef<Path> + Send + Sync> ConfigLoader for YamlConfigLoader<P> {
    #[tracing::instrument(skip(self))]
    fn load(&self) -> Result<RawssgConfig, RawssgError> {
        let content = std::fs::read_to_string(self.path.as_ref())
            .map_err(|e| RawssgError::Config(format!("cannot read config: {}", e)))?;
        serde_yaml::from_str(&content)
            .map_err(|e| RawssgError::Config(format!("invalid config YAML: {}", e)))
    }
}

pub struct DefaultConfig;

impl ConfigLoader for DefaultConfig {
    fn load(&self) -> Result<RawssgConfig, RawssgError> {
        Ok(RawssgConfig::default())
    }
}
