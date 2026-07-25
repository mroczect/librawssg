use librawssg::fs::real::RealFs;
use librawssg::markdown::PulldownMarkdown;
use librawssg::site::builder::SiteBuilder;
use librawssg::site::page::build_page_context;
use librawssg::site::TeraRenderer;
use std::path::Path;
use tracing_subscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let builder = SiteBuilder::new()
        .load_config("config.yaml")
        .expect("Failed to load config.yaml");

    let mut tera = TeraRenderer::new();
    let templates_dir = Path::new("templates");
    for entry in std::fs::read_dir(templates_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let content = std::fs::read_to_string(&path)?;
            tera.add_raw_template(&name, &content)?;
        }
    }

    let md = PulldownMarkdown;
    let content_dir = Path::new("content");
    let output_dir = Path::new("../dist");

    // --- DEBUG: Coba proses satu file secara manual ---
    let fs = RealFs;
    let test_file = content_dir.join("index.md");
    println!("Manual build_page_context for {:?}...", test_file);
    match build_page_context(&fs, &md, &test_file, content_dir) {
        Ok(Some(ctx)) => println!("  Success: title='{}', url='{}'", ctx.frontmatter.title, ctx.url),
        Ok(None) => println!("  Skipped (draft?)"),
        Err(e) => println!("  Error: {}", e),
    }
    // --- Akhir debug ---

    let site = builder
        .content_dir(content_dir)
        .output_dir(output_dir)
        .with_template_renderer(Box::new(tera))
        .with_markdown_renderer(Box::new(md))
        .build()?;

    println!("Number of pages: {}", site.pages().len());
    for page in site.pages() {
        println!("  {} -> {}", page.file_path, page.url);
    }

    site.generate()?;
    println!("Site generated successfully in {}", output_dir.display());

    Ok(())
}
