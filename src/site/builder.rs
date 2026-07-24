use crate::config::ConfigLoader;
use crate::error::RawssgError;
use crate::fs::FileSystem;
use crate::markdown::MarkdownRenderer;
use crate::site::{ContentHandler, MarkdownPageHandler, StaticFileHandler, TemplateRenderer};
use crate::types::{PageContext, RawssgConfig};
use crate::util::{relative_prefix, safe_path};
use std::path::{Path, PathBuf};
use tera::Context;

pub struct Site {
    config: RawssgConfig,
    pages: Vec<PageContext>,
    output_dir: PathBuf,
    base_url: String,
    fs: Box<dyn FileSystem>,
    renderer: Box<dyn TemplateRenderer>,
    content_dir: PathBuf,
}

impl Site {
    pub fn pages(&self) -> &[PageContext] {
        &self.pages
    }

    #[tracing::instrument(skip(self))]
    pub fn generate(self) -> Result<(), RawssgError> {
        let tmp_dir = self.output_dir.with_extension("tmp");
        if self.fs.exists(&tmp_dir) {
            self.fs.remove_dir_all(&tmp_dir)?;
        }
        self.fs.create_dir_all(&tmp_dir)?;

        self.generate_to(&tmp_dir)?;

        if self.fs.exists(&self.output_dir) {
            self.fs.remove_dir_all(&self.output_dir)?;
        }
        std::fs::rename(&tmp_dir, &self.output_dir)
            .map_err(|e| RawssgError::SiteGeneration(format!("Atomic rename failed: {}", e)))?;

        Ok(())
    }

    fn generate_to(&self, output_base: &Path) -> Result<(), RawssgError> {
        self.fs.create_dir_all(output_base)?;

        for page in &self.pages {
            if page.is_list {
                continue;
            }
            let mut ctx = Context::new();
            self.fill_context(page, &mut ctx);
            let template = self.template_for_page(page);
            let html = self.renderer.render(&template, &ctx)?;

            let candidate = Path::new(&page.url);
            let out_path = safe_path(self.fs.as_ref(), output_base, candidate)?;
            if let Some(parent) = out_path.parent() {
                self.fs.create_dir_all(parent)?;
            }
            self.fs.write(&out_path, html.as_bytes())?;
        }

        for page in &self.pages {
            if !page.is_list {
                continue;
            }
            let mut ctx = Context::new();
            self.fill_context(page, &mut ctx);
            if let Some(items) = &page.list_items {
                ctx.insert("pages", items);
            }
            let template = self.template_for_page(page);
            let html = self.renderer.render(&template, &ctx)?;

            let candidate = Path::new(&page.url);
            let out_path = safe_path(self.fs.as_ref(), output_base, candidate)?;
            if let Some(parent) = out_path.parent() {
                self.fs.create_dir_all(parent)?;
            }
            self.fs.write(&out_path, html.as_bytes())?;
        }

        let static_dir = Path::new(&self.config.build.static_dir);
        if self.fs.exists(static_dir) {
            self.copy_static_assets(static_dir, output_base)?;
        }

        self.copy_content_assets(output_base)?;

        if self.config.generators.rss.enabled {
            let blog_posts: Vec<&PageContext> = self
                .pages
                .iter()
                .filter(|p| p.content_type == "blog" && !p.is_list)
                .collect();
            if !blog_posts.is_empty() {
                let rss = crate::site::feed::generate_feed(
                    &*self.renderer,
                    &self.config,
                    &blog_posts,
                    &self.base_url,
                )?;
                let feed_path = safe_path(
                    self.fs.as_ref(),
                    output_base,
                    Path::new(&self.config.generators.rss.path),
                )?;
                if let Some(parent) = feed_path.parent() {
                    self.fs.create_dir_all(parent)?;
                }
                self.fs.write(&feed_path, rss.as_bytes())?;
            }
        }

        if self.config.generators.sitemap.enabled {
            let sitemap = crate::site::sitemap::generate_sitemap(
                &*self.renderer,
                &self.pages,
                &self.base_url,
            )?;
            let sitemap_path = safe_path(
                self.fs.as_ref(),
                output_base,
                Path::new(&self.config.generators.sitemap.path),
            )?;
            if let Some(parent) = sitemap_path.parent() {
                self.fs.create_dir_all(parent)?;
            }
            self.fs.write(&sitemap_path, sitemap.as_bytes())?;
        }

        Ok(())
    }

    fn fill_context(&self, page: &PageContext, ctx: &mut Context) {
        let site = &self.config.site;
        ctx.insert("site_name", &site.site_name);
        ctx.insert(
            "site_description",
            &site.description.as_deref().unwrap_or(""),
        );
        ctx.insert("site_language", &site.language.as_deref().unwrap_or("en"));
        ctx.insert("site_author", &site.author.as_deref().unwrap_or(""));
        ctx.insert("site_license", &site.license.as_deref().unwrap_or(""));
        ctx.insert("base_url", &self.base_url);
        ctx.insert("base_path", &relative_prefix(page.depth));
        ctx.insert("page_title", &page.frontmatter.title);
        ctx.insert("page_desc", &page.frontmatter.desc);
        ctx.insert("page_content", &page.content_html);
        ctx.insert(
            "page_author",
            &page.frontmatter.author.as_deref().unwrap_or(""),
        );
        ctx.insert(
            "page_repo_url",
            &page.frontmatter.repo_url.as_deref().unwrap_or(""),
        );
        ctx.insert(
            "page_license",
            &page.frontmatter.license.as_deref().unwrap_or(""),
        );
        ctx.insert("page_url", &page.url);
        ctx.insert("page_pub_date", &page.pub_date.as_deref().unwrap_or(""));
    }

    fn template_for_page(&self, page: &PageContext) -> String {
        for ct in &self.config.content_types {
            if ct.name == page.content_type {
                return ct.template.clone();
            }
        }
        "base.html".into()
    }

    #[tracing::instrument(skip(self, output_base))]
    fn copy_static_assets(&self, static_dir: &Path, output_base: &Path) -> Result<(), RawssgError> {
        for entry in self.fs.walk_dir(static_dir)? {
            let rel = entry
                .strip_prefix(static_dir)
                .map_err(|e| RawssgError::SiteGeneration(e.to_string()))?;
            let dest = safe_path(self.fs.as_ref(), output_base, rel)?;
            if let Some(parent) = dest.parent() {
                self.fs.create_dir_all(parent)?;
            }
            self.fs.copy_file(&entry, &dest)?;
        }
        Ok(())
    }

    fn copy_content_assets(&self, output_base: &Path) -> Result<(), RawssgError> {
        if !self.fs.exists(&self.content_dir) {
            return Ok(());
        }

        for entry in self.fs.walk_dir(&self.content_dir)? {
            if entry.extension().map(|e| e == "md").unwrap_or(false) {
                continue;
            }

            let rel = entry
                .strip_prefix(&self.content_dir)
                .map_err(|e| RawssgError::SiteGeneration(e.to_string()))?;
            let dest = safe_path(self.fs.as_ref(), output_base, rel)?;
            if let Some(parent) = dest.parent() {
                self.fs.create_dir_all(parent)?;
            }
            self.fs.copy_file(&entry, &dest)?;
        }

        Ok(())
    }
}

pub struct SiteBuilder {
    config: RawssgConfig,
    content_dir: PathBuf,
    output_dir: PathBuf,
    fs: Box<dyn FileSystem>,
    md_renderer: Box<dyn MarkdownRenderer>,
    renderer: Box<dyn TemplateRenderer>,
    handlers: Vec<Box<dyn ContentHandler>>,
}

impl SiteBuilder {
    pub fn new() -> Self {
        Self {
            config: RawssgConfig::default(),
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("dist"),
            fs: Box::new(crate::fs::real::RealFs),
            md_renderer: Box::new(crate::markdown::PulldownMarkdown),
            renderer: Box::new(crate::site::TeraRenderer::new()),
            handlers: vec![Box::new(MarkdownPageHandler), Box::new(StaticFileHandler)],
        }
    }

    pub fn config(mut self, config: RawssgConfig) -> Self {
        self.config = config;
        self
    }
    pub fn load_config<P: AsRef<Path> + Send + Sync>(
        mut self,
        path: P,
    ) -> Result<Self, RawssgError> {
        let loader = crate::config::loader::YamlConfigLoader::new(path);
        self.config = loader.load()?;
        Ok(self)
    }
    pub fn content_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.content_dir = dir.into();
        self
    }
    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }
    pub fn with_fs(mut self, fs: Box<dyn FileSystem>) -> Self {
        self.fs = fs;
        self
    }
    pub fn with_markdown_renderer(mut self, md: Box<dyn MarkdownRenderer>) -> Self {
        self.md_renderer = md;
        self
    }
    pub fn with_template_renderer(mut self, tr: Box<dyn TemplateRenderer>) -> Self {
        self.renderer = tr;
        self
    }
    pub fn add_handler(mut self, handler: Box<dyn ContentHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    #[tracing::instrument(skip(self))]
    pub fn build(mut self) -> Result<Site, RawssgError> {
        self.config.validate()?;

        if self.content_dir == PathBuf::from("content") {
            self.content_dir = PathBuf::from(&self.config.build.content_dir);
        }
        if self.output_dir == PathBuf::from("dist") {
            self.output_dir = PathBuf::from(&self.config.build.output_dir);
        }

        let templates_dir = PathBuf::from(&self.config.build.templates_dir);
        if self.fs.exists(&templates_dir) {
            for entry in self.fs.walk_dir(&templates_dir)? {
                if let Ok(rel) = entry.strip_prefix(&templates_dir) {
                    let name = rel.to_string_lossy().to_string();
                    let content = self.fs.read_to_string(&entry)?;
                    self.renderer.add_raw_template(&name, &content)?;
                }
            }
        }

        let base_url = self
            .config
            .site
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:3000".to_string());

        let mut pages = Vec::new();
        let mut blog_posts = Vec::new();

        let all_files = self.fs.walk_dir(&self.content_dir)?;
        for file_path in &all_files {
            let rel = match file_path.strip_prefix(&self.content_dir) {
                Ok(r) => r.to_path_buf(),
                Err(_) => continue,
            };
            let mut handled = false;
            for handler in &self.handlers {
                if handler.can_handle(&rel, file_path) {
                    match handler.process(
                        &*self.fs,
                        &*self.md_renderer,
                        file_path,
                        &self.content_dir,
                    ) {
                        Ok(Some(ctx)) => {
                            let mut ctx = ctx;
                            ctx.content_type = self.determine_content_type(&rel);
                            if ctx.content_type == "blog" && !ctx.is_list {
                                blog_posts.push(ctx.clone());
                            }
                            pages.push(ctx);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            tracing::error!("Failed to process {}: {}", file_path.display(), e);
                        }
                    }
                    handled = true;
                    break;
                }
            }
            if !handled {
                tracing::warn!("No handler for: {}", file_path.display());
            }
        }

        blog_posts.sort_by(|a, b| {
            b.frontmatter
                .date
                .cmp(&a.frontmatter.date)
                .then_with(|| a.frontmatter.title.cmp(&b.frontmatter.title))
        });

        for ct in &self.config.content_types {
            if ct.list_enabled && ct.list_template.is_some() {
                let items: Vec<PageContext> = pages
                    .iter()
                    .filter(|p| p.content_type == ct.name && !p.is_list)
                    .cloned()
                    .collect();
                if !items.is_empty() {
                    let list_url = format!("{}/index.html", ct.name);
                    let list_page = PageContext {
                        url: list_url,
                        file_path: String::new(),
                        depth: 1,
                        pub_date: None,
                        frontmatter: Default::default(),
                        content_html: String::new(),
                        content_type: ct.name.clone(),
                        is_list: true,
                        list_items: Some(items),
                    };
                    pages.push(list_page);
                }
            }
        }

        Ok(Site {
            config: self.config,
            pages,
            output_dir: self.output_dir,
            base_url,
            fs: self.fs,
            renderer: self.renderer,
            content_dir: self.content_dir,
        })
    }

    fn determine_content_type(&self, relative_path: &Path) -> String {
        for ct in &self.config.content_types {
            if crate::util::match_pattern(&ct.pattern, relative_path) {
                return ct.name.clone();
            }
        }
        "page".into()
    }
}
