use crate::error::RawssgError;
use crate::types::GlobalConfig;

pub trait ConfigLoader: Send + Sync {
    fn load(&self) -> Result<GlobalConfig, RawssgError>;

    fn load_or_default(&self) -> GlobalConfig {
        self.load().unwrap_or_default()
    }
}

pub mod loader;
