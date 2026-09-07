use crate::pipeline::Pipeline;
use librawssg_error::Result;
use std::path::Path;

pub trait Generator: Send + Sync {
    fn generate(&self, pipeline: &Pipeline, output_base: &Path) -> Result<()>;
}
