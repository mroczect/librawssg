use librawssg::markdown::PulldownMarkdown;
use librawssg::site::context::{TeraFeedContextBuilder, TeraSitemapContextBuilder};
use librawssg::site::TeraRenderer;
use librawssg::SiteBuilder;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let builder = SiteBuilder::new()
        .load_config("config.yaml")
        .expect("Gagal memuat config.yaml");

    let mut tera = TeraRenderer::new();
    let templates_dir = Path::new("templates");
    if templates_dir.exists() {
        for entry in std::fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                let content = std::fs::read_to_string(&path)?;
                tera.add_raw_template(&name, &content)?;
            }
        }
    }

    let md = PulldownMarkdown;
    let content_dir = Path::new("content");
    let output_dir = Path::new("./dist");

    let site = builder
        .content_dir(content_dir)
        .output_dir(output_dir)
        .with_template_renderer(Box::new(tera))
        .with_markdown_renderer(Box::new(md))
        .with_feed_context_builder(Box::new(TeraFeedContextBuilder))
        .with_sitemap_context_builder(Box::new(TeraSitemapContextBuilder))
        .build()?;

    println!("Jumlah halaman: {}", site.pages().len());
    for page in site.pages() {
        println!("  {} -> {}", page.file_path, page.url);
    }

    site.generate()?;
    println!("Situs berhasil dibuat di {}", output_dir.display());

    Ok(())
}
