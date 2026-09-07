#![allow(clippy::multiple_crate_versions)]

pub mod builder;
pub mod context;
pub mod generator;
pub mod pattern;
pub mod pipeline;

pub use builder::PipelineBuilder;
pub use context::ContextBuilder;
pub use generator::Generator;
pub use pipeline::Pipeline;

#[cfg(feature = "tera")]
pub use context::TeraContextBuilder;
