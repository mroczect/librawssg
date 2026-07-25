use librawssg::markdown::PulldownMarkdown;
use librawssg::site::builder::SiteBuilder;
use librawssg::site::TeraRenderer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 1. Muat konfigurasi
    let builder = SiteBuilder::new()
        .load_config("config.yaml")
        .expect("Gagal memuat config.yaml");

    // 2. Siapkan template engine dan muat semua template
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

    // 3. Markdown renderer
    let md = PulldownMarkdown;

    // 4. Direktori konten dan output
    let content_dir = Path::new("content");
    let output_dir = Path::new("../dist");

    // 5. Bangun site
    let site = builder
        .content_dir(content_dir)
        .output_dir(output_dir)
        .with_template_renderer(Box::new(tera))
        .with_markdown_renderer(Box::new(md))
        .build()?;

    // 6. Tampilkan info halaman
    println!("Jumlah halaman: {}", site.pages().len());
    for page in site.pages() {
        println!("  {} -> {}", page.file_path, page.url);
    }

    // 7. Generate
    site.generate()?;
    println!("Situs berhasil dibuat di {}", output_dir.display());

    Ok(())
}
