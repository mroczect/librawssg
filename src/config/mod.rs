pub mod loader;

use crate::error::RawssgError;
use crate::types::RawssgConfig;

pub trait ConfigLoader: Send + Sync {
    fn load(&self) -> Result<RawssgConfig, RawssgError>;
    fn load_or_default(&self) -> RawssgConfig {
        self.load().unwrap_or_default()
    }
}
