use core::any::Any;
use librawssg_error::Result;

pub trait RenderContext: Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn as_mut_any(&mut self) -> &mut dyn Any;
}

pub trait Renderer: Send + Sync {
    fn render(&self, template_name: &str, context: &dyn RenderContext) -> Result<String>;
}
