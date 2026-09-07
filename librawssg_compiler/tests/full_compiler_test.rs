use librawssg_compiler::{ContextBuilder, Generator, Pipeline, PipelineBuilder};
use librawssg_config::{Config, ContentRule};
use librawssg_fs::{FileSystem, RealFs};
use librawssg_handler::{Document, Metadata, Processor};
use librawssg_templates::{RenderContext, Renderer};
use std::path::{Path, PathBuf};
use tempfile as _;
use tera as _;

macro_rules! must {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                eprintln!("{} failed: {}", $context, err);
                std::process::exit(1);
            }
        }
    };
}

struct MockRenderer;

impl Renderer for MockRenderer {
    fn render(
        &self,
        template_name: &str,
        _context: &dyn RenderContext,
    ) -> librawssg_error::Result<String> {
        Ok(format!("rendered:{template_name}"))
    }
}

struct MockContext;

impl RenderContext for MockContext {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

struct MockContextBuilder;

impl ContextBuilder for MockContextBuilder {
    fn build_context(
        &self,
        _config: &Config,
        _doc: &Document,
    ) -> librawssg_error::Result<Box<dyn RenderContext>> {
        Ok(Box::new(MockContext))
    }
}

struct RawHtmlProcessor;

impl Processor for RawHtmlProcessor {
    fn name(&self) -> &'static str {
        "raw-html"
    }

    fn can_process(&self, relative_path: &Path, _original_path: &Path) -> bool {
        relative_path.extension().is_some_and(|ext| ext == "html")
    }

    fn process(
        &self,
        fs: &dyn FileSystem,
        relative_path: &Path,
        content_dir: &Path,
    ) -> librawssg_error::Result<Option<Document>> {
        let full_path = content_dir.join(relative_path);
        let content = fs.read_to_string(&full_path)?;
        let url = relative_path
            .with_extension("html")
            .to_string_lossy()
            .to_string();
        let output_path = PathBuf::from(&url);
        let metadata = Metadata::new("Test", "Description")?;
        let doc = Document::new(
            metadata,
            content,
            url,
            output_path,
            relative_path.to_path_buf(),
            0,
            "page".to_string(),
            false,
        )?;
        Ok(Some(doc))
    }
}

struct DummyGenerator;

impl DummyGenerator {
    #[must_use]
    const fn new() -> Self {
        Self
    }
}

impl Generator for DummyGenerator {
    fn generate(&self, _pipeline: &Pipeline, output_base: &Path) -> librawssg_error::Result<()> {
        let path = output_base.join("generated.txt");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| librawssg_error::Error::Generation(e.to_string()))?;
        }
        std::fs::write(&path, b"generated")
            .map_err(|e| librawssg_error::Error::Generation(e.to_string()))
    }
}
fn setup_config() -> Config {
    let mut config = Config::new().with_site_name("Compiler Test");
    config.add_content_rule(ContentRule::new("page", "**/*.html", "base"));
    config
}

fn build_basic_pipeline(
    content_dir: &Path,
    output_dir: &Path,
    config: Config,
) -> librawssg_error::Result<Pipeline> {
    PipelineBuilder::new()
        .config(config)
        .content_dir(content_dir)
        .output_dir(output_dir)
        .with_fs(Box::new(RealFs))
        .with_renderer(Box::new(MockRenderer))
        .with_context_builder(Box::new(MockContextBuilder))
        .add_processor(Box::new(RawHtmlProcessor))
        .build()
}

#[test]
fn pipeline_generates_single_page() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(std::fs::create_dir_all(&content_dir), "create content dir");
    must!(
        std::fs::write(content_dir.join("index.html"), "<h1>Home</h1>"),
        "write index.html"
    );

    let pipeline = must!(
        build_basic_pipeline(&content_dir, &output_dir, setup_config()),
        "build pipeline"
    );
    must!(pipeline.run(), "run pipeline");

    let output_file = output_dir.join("index.html");
    assert!(output_file.exists());
    let content = must!(std::fs::read_to_string(output_file), "read output");
    assert_eq!(content, "rendered:base");
}

#[test]
fn pipeline_respects_content_type_rules() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(
        std::fs::create_dir_all(content_dir.join("blog")),
        "create blog dir"
    );
    must!(
        std::fs::write(content_dir.join("blog/post.html"), "post"),
        "write post.html"
    );
    must!(
        std::fs::write(content_dir.join("index.html"), "index"),
        "write index.html"
    );

    let mut config = setup_config();
    config.add_content_rule(ContentRule::new("blog", "blog/**/*.html", "post_template"));

    let pipeline = must!(
        build_basic_pipeline(&content_dir, &output_dir, config),
        "build pipeline"
    );
    must!(pipeline.run(), "run pipeline");

    let blog_output = output_dir.join("blog/post.html");
    assert!(blog_output.exists());
    let blog_content = must!(std::fs::read_to_string(blog_output), "read blog output");
    assert_eq!(blog_content, "rendered:post_template");

    let index_output = output_dir.join("index.html");
    assert!(index_output.exists());
    let index_content = must!(std::fs::read_to_string(index_output), "read index output");
    assert_eq!(index_content, "rendered:base");
}

#[test]
fn pipeline_generates_list_pages() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(
        std::fs::create_dir_all(content_dir.join("blog")),
        "create blog dir"
    );
    must!(
        std::fs::write(content_dir.join("blog/one.html"), "one"),
        "write one.html"
    );
    must!(
        std::fs::write(content_dir.join("blog/two.html"), "two"),
        "write two.html"
    );

    let mut config = setup_config();
    let mut blog_rule = ContentRule::new("blog", "blog/**/*.html", "post");
    blog_rule.list_template = Some("list".into());
    blog_rule.list_enabled = true;
    config.add_content_rule(blog_rule);

    let pipeline = must!(
        build_basic_pipeline(&content_dir, &output_dir, config),
        "build pipeline"
    );
    must!(pipeline.run(), "run pipeline");

    let list_output = output_dir.join("blog/index.html");
    assert!(list_output.exists());
    let list_content = must!(std::fs::read_to_string(list_output), "read list output");
    assert_eq!(list_content, "rendered:list");
}

#[test]
fn pipeline_copies_static_assets() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    let static_dir = tmp.path().join("static");
    must!(std::fs::create_dir_all(&content_dir), "create content dir");
    must!(std::fs::create_dir_all(&static_dir), "create static dir");
    must!(
        std::fs::write(content_dir.join("index.html"), "home"),
        "write index.html"
    );
    must!(
        std::fs::write(static_dir.join("style.css"), "body {}"),
        "write style.css"
    );

    let mut config = setup_config();
    config.build.static_dir = static_dir.to_string_lossy().to_string();

    let pipeline = must!(
        build_basic_pipeline(&content_dir, &output_dir, config),
        "build pipeline"
    );
    must!(pipeline.run(), "run pipeline");

    let style_output = output_dir.join(static_dir).join("style.css");
    assert!(style_output.exists());
    let style_content = must!(std::fs::read_to_string(style_output), "read style");
    assert_eq!(style_content, "body {}");
}

#[test]
fn pipeline_runs_generators() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(std::fs::create_dir_all(&content_dir), "create content dir");
    must!(
        std::fs::write(content_dir.join("index.html"), "hello"),
        "write index.html"
    );

    let generator = DummyGenerator::new();
    let pipeline = must!(
        PipelineBuilder::new()
            .config(setup_config())
            .content_dir(&content_dir)
            .output_dir(&output_dir)
            .with_fs(Box::new(RealFs))
            .with_renderer(Box::new(MockRenderer))
            .with_context_builder(Box::new(MockContextBuilder))
            .add_processor(Box::new(RawHtmlProcessor))
            .add_generator(Box::new(generator))
            .build(),
        "build pipeline"
    );
    must!(pipeline.run(), "run pipeline");

    assert!(output_dir.join("generated.txt").exists());
}

#[test]
fn pipeline_atomic_replace_existing_output() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(std::fs::create_dir_all(&content_dir), "create content dir");
    must!(
        std::fs::write(content_dir.join("index.html"), "new"),
        "write index.html"
    );

    must!(std::fs::create_dir_all(&output_dir), "create output dir");
    must!(
        std::fs::write(output_dir.join("old.txt"), "old"),
        "write old file"
    );

    let pipeline = must!(
        build_basic_pipeline(&content_dir, &output_dir, setup_config()),
        "build pipeline"
    );
    must!(pipeline.run(), "run pipeline");

    assert!(!output_dir.join("old.txt").exists());
    assert!(output_dir.join("index.html").exists());
    let content = must!(
        std::fs::read_to_string(output_dir.join("index.html")),
        "read new"
    );
    assert_eq!(content, "rendered:base");
}

#[test]
fn pipeline_handles_empty_content_dir() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(std::fs::create_dir_all(&content_dir), "create content dir");

    let pipeline = must!(
        build_basic_pipeline(&content_dir, &output_dir, setup_config()),
        "build pipeline"
    );
    must!(pipeline.run(), "run pipeline");

    assert!(output_dir.exists());
    let entries = must!(std::fs::read_dir(&output_dir), "read output dir");
    assert_eq!(entries.count(), 0);
}

#[test]
fn pipeline_skips_files_not_matching_processor() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(std::fs::create_dir_all(&content_dir), "create content dir");
    must!(
        std::fs::write(content_dir.join("index.html"), "home"),
        "write index.html"
    );
    must!(
        std::fs::write(content_dir.join("script.js"), "console.log('hi')"),
        "write script.js"
    );

    let pipeline = must!(
        build_basic_pipeline(&content_dir, &output_dir, setup_config()),
        "build pipeline"
    );
    must!(pipeline.run(), "run pipeline");

    assert!(output_dir.join("index.html").exists());
    assert!(!output_dir.join("script.js").exists());
}

#[test]
fn pipeline_errors_when_renderer_missing() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(std::fs::create_dir_all(&content_dir), "create content dir");

    let result = PipelineBuilder::new()
        .config(setup_config())
        .content_dir(&content_dir)
        .output_dir(&output_dir)
        .with_fs(Box::new(RealFs))
        .with_context_builder(Box::new(MockContextBuilder))
        .add_processor(Box::new(RawHtmlProcessor))
        .build();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, librawssg_error::Error::Config(_)));
    }
}

#[test]
fn pipeline_errors_when_context_builder_missing() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");
    must!(std::fs::create_dir_all(&content_dir), "create content dir");

    let result = PipelineBuilder::new()
        .config(setup_config())
        .content_dir(&content_dir)
        .output_dir(&output_dir)
        .with_fs(Box::new(RealFs))
        .with_renderer(Box::new(MockRenderer))
        .add_processor(Box::new(RawHtmlProcessor))
        .build();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, librawssg_error::Error::Config(_)));
    }
}

#[test]
fn pipeline_errors_on_invalid_config() {
    let tmp = must!(tempfile::TempDir::new(), "TempDir::new");
    let content_dir = tmp.path().join("content");
    let output_dir = tmp.path().join("dist");

    let mut config = setup_config();
    config.site.site_name = " ".to_string();

    let result = PipelineBuilder::new()
        .config(config)
        .content_dir(&content_dir)
        .output_dir(&output_dir)
        .with_fs(Box::new(RealFs))
        .with_renderer(Box::new(MockRenderer))
        .with_context_builder(Box::new(MockContextBuilder))
        .add_processor(Box::new(RawHtmlProcessor))
        .build();
    assert!(result.is_err());
}
