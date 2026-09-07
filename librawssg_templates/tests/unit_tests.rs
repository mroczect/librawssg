use core::any::Any;
use librawssg_error::Result;
use librawssg_templates::{RenderContext, Renderer};
use tempfile as _;
use tera as _;
use walkdir as _;
struct MockRenderer {
    output: String,
}

impl Renderer for MockRenderer {
    fn render(&self, _template_name: &str, _context: &dyn RenderContext) -> Result<String> {
        Ok(self.output.clone())
    }
}

struct MockContext;

impl RenderContext for MockContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[test]
fn renderer_trait_can_be_mocked() {
    let renderer = MockRenderer {
        output: "mock".to_string(),
    };
    let ctx = MockContext;
    let output = match renderer.render("whatever", &ctx) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("render failed: {}", e);
            std::process::exit(1);
        }
    };
    assert_eq!(output, "mock");
}

#[test]
fn render_context_trait_can_be_mocked() {
    let ctx = MockContext;
    let dyn_ctx: &dyn RenderContext = &ctx;
    assert!(dyn_ctx.as_any().is::<MockContext>());
}

#[test]
fn reexports_available() {
    let _: Option<&dyn Renderer> = None;
    let _: Option<&dyn RenderContext> = None;
}
