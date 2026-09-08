#![allow(clippy::multiple_crate_versions)]

use librawssg::{
    Config, ContentRule, Document, FileSystem, Metadata, NavItem, PipelineBuilder, Processor,
    RealFs, TeraContextBuilder, TeraRenderer,
};
use std::path::{Path, PathBuf};

struct RawFileProcessor;

impl Processor for RawFileProcessor {
    fn name(&self) -> &'static str {
        "raw-file"
    }

    fn can_process(&self, relative_path: &Path, _original_path: &Path) -> bool {
        relative_path.extension().is_some_and(|ext| ext == "raw")
    }

    fn process(
        &self,
        fs: &dyn FileSystem,
        relative_path: &Path,
        content_dir: &Path,
    ) -> librawssg::Result<Option<Document>> {
        let full_path = content_dir.join(relative_path);
        let body = fs.read_to_string(&full_path)?;

        let stem = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled");
        let title = stem
            .split('-')
            .map(|word| {
                let mut c = word.chars();
                c.next().map_or_else(String::new, |first| {
                    first.to_uppercase().collect::<String>() + c.as_str()
                })
            })
            .collect::<Vec<_>>()
            .join(" ");

        let metadata = Metadata::new(title, String::new())?;
        let url = relative_path
            .with_extension("html")
            .to_string_lossy()
            .to_string();
        let output_path = PathBuf::from(&url);

        let doc = Document::new(
            metadata,
            body,
            url,
            output_path,
            relative_path.to_path_buf(),
            relative_path.components().count().saturating_sub(1),
            "page".to_string(),
            false,
        )?;
        Ok(Some(doc))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    let content_dir = base.join("src/content");
    let templates_dir = base.join("src/templates");
    let static_dir = base.join("src/static");
    let output_dir = base.join("dist");

    let mut renderer = TeraRenderer::new();
    renderer.add_template_file(&templates_dir.join("macros.tera"))?;
    renderer.load_templates_dir(&templates_dir)?;

    let mut config = Config::new().with_site_name("librawssg Docs");

    config.add_content_rule(ContentRule::new("page", "**/*.raw", "base.tera"));

    config.build.content_dir = content_dir.to_string_lossy().to_string();
    config.build.output_dir = output_dir.to_string_lossy().to_string();
    config.build.static_dir = static_dir.to_string_lossy().to_string();

    config.site.navbar = vec![
        NavItem::new("Home", "/"),
        NavItem::new("Installation", "/installation.html"),
        NavItem::new("Configuration", "/configuration.html"),
        NavItem::new("API", "/api/index.html"),
        NavItem::new("Contributing", "/contributing.html"),
    ];

    let mut api_item = NavItem::new("API Reference", "/api/index.html");
    api_item.children = vec![
        NavItem::new("Compiler", "/api/compiler.html"),
        NavItem::new("Config", "/api/config.html"),
        NavItem::new("Error", "/api/error.html"),
        NavItem::new("Filesystem", "/api/fs.html"),
        NavItem::new("Handler", "/api/handler.html"),
        NavItem::new("Templates", "/api/templates.html"),
    ];

    config.site.sidebar = vec![
        NavItem::new("Getting Started", "/"),
        NavItem::new("Installation", "/installation.html"),
        NavItem::new("Configuration", "/configuration.html"),
        api_item,
        NavItem::new("Contributing", "/contributing.html"),
        NavItem::new("Code of Conduct", "/code_of_conduct.html"),
        NavItem::new("License", "/license.html"),
    ];

    let pipeline = PipelineBuilder::new()
        .config(config)
        .content_dir(&content_dir)
        .output_dir(&output_dir)
        .with_fs(Box::new(RealFs))
        .with_renderer(Box::new(renderer))
        .with_context_builder(Box::new(TeraContextBuilder))
        .add_processor(Box::new(RawFileProcessor))
        .build()?;

    pipeline.run()?;

    println!("✅ Docs generated in '{}'", output_dir.display());
    Ok(())
}
