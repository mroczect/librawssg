use crate::pipeline::Pipeline;
use librawssg_error::Result;

pub trait Generator: Send + Sync {
    fn generate(&self, pipeline: &Pipeline) -> Result<()>;
}
