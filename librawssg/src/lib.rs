#![allow(clippy::multiple_crate_versions)]

pub mod config {
    pub use librawssg_config::*;
}
pub mod fs {
    pub use librawssg_fs::*;
}
pub mod handler {
    pub use librawssg_handler::*;
}
pub mod templates {
    pub use librawssg_templates::*;
}
pub mod compiler {
    pub use librawssg_compiler::*;
}
pub mod error {
    pub use librawssg_error::*;
}

pub use librawssg_compiler::{
    ContextBuilder, Generator, Pipeline, PipelineBuilder, TeraContextBuilder,
};
pub use librawssg_config::{BuildConfig, Config, ContentRule, NavItem, SiteConfig};
pub use librawssg_error::{Error, Result};
pub use librawssg_fs::{FileSystem, RealFs};
pub use librawssg_handler::{Document, Metadata, Processor};
pub use librawssg_templates::{RenderContext, Renderer, TeraRenderer};
