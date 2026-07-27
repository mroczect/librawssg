use crate::config::ConfigLoader;
use crate::error::RawssgError;
use crate::fs::FileSystem;
use crate::markdown::MarkdownRenderer;
#[cfg(feature = "tera")]
use crate::site::context::{FeedContextBuilder, SitemapContextBuilder};
use crate::site::{ContentHandler, MarkdownPageHandler, StaticFileHandler, TemplateRenderer};
use crate::types::{PageContext, RawssgConfig};
use std::path::{Path, PathBuf};

use super::site::Site;

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
