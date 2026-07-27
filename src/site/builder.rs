use crate::config::ConfigLoader;
use crate::error::RawssgError;
use crate::fs::FileSystem;
use crate::markdown::MarkdownRenderer;
#[cfg(feature = "tera")]
use crate::site::context::{FeedContextBuilder, SitemapContextBuilder};
use crate::site::{
    ContentHandler, Context, MarkdownPageHandler, StaticFileHandler, TemplateRenderer,
};
use crate::types::{PageContext, RawssgConfig};
#[cfg(feature = "tera")]
use crate::util::relative_prefix;
use crate::util::safe_path;
use std::io;
use std::path::{Path, PathBuf};

pub struct Site {
    config: RawssgConfig,
    pages: Vec<PageContext>,
    output_dir: PathBuf,
    #[cfg_attr(not(feature = "tera"), allow(dead_code))]
    base_url: String,
    fs: Box<dyn FileSystem>,
    renderer: Box<dyn TemplateRenderer>,
    content_dir: PathBuf,
    #[cfg(feature = "tera")]
    feed_context_builder: Option<Box<dyn FeedContextBuilder>>,
    #[cfg(feature = "tera")]
    sitemap_context_builder: Option<Box<dyn SitemapContextBuilder>>,
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
        std::fs::rename(&tmp_dir, &self.output_dir).or_else(|e| {
            if e.kind() == io::ErrorKind::CrossesDevices {
                self.copy_dir_all(&tmp_dir, &self.output_dir)?;
                self.fs.remove_dir_all(&tmp_dir)?;
                Ok(())
            } else {
                Err(RawssgError::SiteGeneration(format!(
                    "Atomic rename failed: {}",
                    e
                )))
            }
        })?;

        Ok(())
    }

    fn generate_to(&self, output_base: &Path) -> Result<(), RawssgError> {
        self.fs.create_dir_all(output_base)?;

        let static_dir = self.try_canonicalize_or_skip(Path::new(&self.config.build.static_dir))?;
        let content_dir = self.try_canonicalize_or_skip(&self.content_dir)?;

        for page in &self.pages {
            if page.is_list {
                continue;
            }
            let ctx = self.build_context(page)?;
            let template = self.template_for_page(page);
            let html = self.renderer.render(&template, &*ctx)?;

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
            #[allow(unused_mut)]
            let mut ctx = self.build_context(page)?;
            if let Some(_items) = &page.list_items {
                #[cfg(feature = "tera")]
                if let Some(tera_ctx) = ctx.as_mut_any().downcast_mut::<tera::Context>() {
                    tera_ctx.insert("pages", _items);
                }
            }
            let template = self.template_for_page(page);
            let html = self.renderer.render(&template, &*ctx)?;

            let candidate = Path::new(&page.url);
            let out_path = safe_path(self.fs.as_ref(), output_base, candidate)?;
            if let Some(parent) = out_path.parent() {
                self.fs.create_dir_all(parent)?;
            }
            self.fs.write(&out_path, html.as_bytes())?;
        }

        if let Some(ref dir) = static_dir
            && self.fs.exists(dir)
        {
            self.copy_static_assets(dir, output_base)?;
        }
        if let Some(ref dir) = content_dir {
            self.copy_content_assets(dir, output_base)?;
        }
        #[cfg(feature = "tera")]
        {
            if self.config.generators.rss.enabled {
                if let Some(ref feed_builder) = self.feed_context_builder {
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
                            &**feed_builder,
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
                } else {
                    return Err(RawssgError::Internal(
                        "RSS enabled but no feed context builder provided".into(),
                    ));
                }
            }

            if self.config.generators.sitemap.enabled {
                if let Some(ref sitemap_builder) = self.sitemap_context_builder {
                    let sitemap = crate::site::sitemap::generate_sitemap(
                        &*self.renderer,
                        &self.config,
                        &self.pages,
                        &self.base_url,
                        &**sitemap_builder,
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
                } else {
                    return Err(RawssgError::Internal(
                        "Sitemap enabled but no sitemap context builder provided".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn try_canonicalize_or_skip(&self, path: &Path) -> Result<Option<PathBuf>, RawssgError> {
        match self.fs.canonicalize(path) {
            Ok(p) if self.fs.exists(&p) => Ok(Some(p)),
            Ok(_) => Ok(None),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(RawssgError::SiteGeneration(format!(
                "Cannot access directory '{}': {}",
                path.display(),
                e
            ))),
        }
    }

    #[cfg(feature = "tera")]
    fn build_context(&self, page: &PageContext) -> Result<Box<dyn Context>, RawssgError> {
        let mut ctx = tera::Context::new();
        self.fill_tera_context(page, &mut ctx);
        Ok(Box::new(ctx))
    }

    #[cfg(not(feature = "tera"))]
    fn build_context(&self, _page: &PageContext) -> Result<Box<dyn Context>, RawssgError> {
        Err(RawssgError::Internal(
            "Context builder not available without 'tera' feature".into(),
        ))
    }

    #[cfg(feature = "tera")]
    fn fill_tera_context(&self, page: &PageContext, ctx: &mut tera::Context) {
        ctx.insert("site", &self.config.site);
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
                if page.is_list
                    && let Some(ref list_tpl) = ct.list_template
                {
                    return list_tpl.clone();
                }
                return ct.template.clone();
            }
        }
        "base.html".into()
    }

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

    fn copy_content_assets(
        &self,
        content_dir: &Path,
        output_base: &Path,
    ) -> Result<(), RawssgError> {
        if !self.fs.exists(content_dir) {
            return Ok(());
        }
        for entry in self.fs.walk_dir(content_dir)? {
            if entry.extension().map(|e| e == "md").unwrap_or(false) {
                continue;
            }
            let rel = entry
                .strip_prefix(content_dir)
                .map_err(|e| RawssgError::SiteGeneration(e.to_string()))?;
            let dest = safe_path(self.fs.as_ref(), output_base, rel)?;
            if let Some(parent) = dest.parent() {
                self.fs.create_dir_all(parent)?;
            }
            self.fs.copy_file(&entry, &dest)?;
        }
        Ok(())
    }

    fn copy_dir_all(&self, from: &Path, to: &Path) -> Result<(), RawssgError> {
        self.fs.create_dir_all(to)?;
        for entry in self.fs.walk_dir(from)? {
            let rel = entry
                .strip_prefix(from)
                .map_err(|e| RawssgError::SiteGeneration(e.to_string()))?;
            let dest = to.join(rel);
            if self.fs.is_dir(&entry) {
                self.fs.create_dir_all(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    self.fs.create_dir_all(parent)?;
                }
                self.fs.copy_file(&entry, &dest)?;
            }
        }
        Ok(())
    }
}

pub struct SiteBuilder {
    config: RawssgConfig,
    content_dir: PathBuf,
    output_dir: PathBuf,
    fs: Box<dyn FileSystem>,
    md_renderer: Option<Box<dyn MarkdownRenderer>>,
    renderer: Option<Box<dyn TemplateRenderer>>,
    handlers: Vec<Box<dyn ContentHandler>>,
    #[cfg(feature = "tera")]
    feed_context_builder: Option<Box<dyn FeedContextBuilder>>,
    #[cfg(feature = "tera")]
    sitemap_context_builder: Option<Box<dyn SitemapContextBuilder>>,
}

impl SiteBuilder {
    pub fn new() -> Self {
        Self {
            config: RawssgConfig::default(),
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("dist"),
            fs: Box::new(crate::fs::real::RealFs),
            md_renderer: None,
            renderer: None,
            handlers: vec![Box::new(MarkdownPageHandler), Box::new(StaticFileHandler)],
            #[cfg(feature = "tera")]
            feed_context_builder: None,
            #[cfg(feature = "tera")]
            sitemap_context_builder: None,
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
        self.md_renderer = Some(md);
        self
    }

    pub fn with_template_renderer(mut self, tr: Box<dyn TemplateRenderer>) -> Self {
        self.renderer = Some(tr);
        self
    }

    pub fn add_handler(mut self, handler: Box<dyn ContentHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    #[cfg(feature = "tera")]
    pub fn with_feed_context_builder(mut self, b: Box<dyn FeedContextBuilder>) -> Self {
        self.feed_context_builder = Some(b);
        self
    }

    #[cfg(feature = "tera")]
    pub fn with_sitemap_context_builder(mut self, b: Box<dyn SitemapContextBuilder>) -> Self {
        self.sitemap_context_builder = Some(b);
        self
    }

    #[tracing::instrument(skip(self))]
    pub fn build(mut self) -> Result<Site, RawssgError> {
        self.config.validate()?;

        let md_renderer = self
            .md_renderer
            .take()
            .ok_or_else(|| RawssgError::Config("markdown renderer not set".into()))?;
        let renderer = self
            .renderer
            .take()
            .ok_or_else(|| RawssgError::Config("template renderer not set".into()))?;

        #[cfg(feature = "tera")]
        let feed_context_builder = if self.config.generators.rss.enabled {
            Some(
                self.feed_context_builder
                    .take()
                    .ok_or_else(|| RawssgError::Config("feed context builder not set".into()))?,
            )
        } else {
            None
        };

        #[cfg(feature = "tera")]
        let sitemap_context_builder = if self.config.generators.sitemap.enabled {
            Some(
                self.sitemap_context_builder
                    .take()
                    .ok_or_else(|| RawssgError::Config("sitemap context builder not set".into()))?,
            )
        } else {
            None
        };

        if self.content_dir == Path::new("content") {
            self.content_dir = PathBuf::from(&self.config.build.content_dir);
        }
        if self.output_dir == Path::new("dist") {
            self.output_dir = PathBuf::from(&self.config.build.output_dir);
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
            for handler in &self.handlers {
                if handler.can_handle(&rel, file_path) {
                    match handler.process(&*self.fs, &*md_renderer, &rel, &self.content_dir) {
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
                    break;
                }
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
                    pages.push(PageContext {
                        url: list_url,
                        file_path: String::new(),
                        depth: 1,
                        pub_date: None,
                        frontmatter: crate::types::PageFrontMatter {
                            title: ct.name.clone(),
                            ..Default::default()
                        },
                        content_html: String::new(),
                        content_type: ct.name.clone(),
                        is_list: true,
                        list_items: Some(items),
                    });
                }
            }
        }

        Ok(Site {
            config: self.config,
            pages,
            output_dir: self.output_dir,
            base_url,
            fs: self.fs,
            renderer,
            content_dir: self.content_dir,
            #[cfg(feature = "tera")]
            feed_context_builder,
            #[cfg(feature = "tera")]
            sitemap_context_builder,
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

impl Default for SiteBuilder {
    fn default() -> Self {
        Self::new()
    }
}
