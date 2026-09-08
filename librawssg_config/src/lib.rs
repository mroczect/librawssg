#![allow(clippy::multiple_crate_versions)]

pub mod build;
pub mod config;
pub mod content_rule;
pub mod nav;
pub mod site;

pub use build::BuildConfig;
pub use config::Config;
pub use content_rule::ContentRule;
pub use nav::NavItem;
pub use site::SiteConfig;

#[cfg(test)]
use tempfile as _;
