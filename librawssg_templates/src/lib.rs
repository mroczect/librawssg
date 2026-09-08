#![allow(clippy::multiple_crate_versions)]

pub mod renderer;
#[cfg(feature = "tera")]
pub mod tera_renderer;

pub use renderer::{RenderContext, Renderer};
#[cfg(feature = "tera")]
pub use tera_renderer::TeraRenderer;

#[cfg(test)]
use tempfile as _;
