use crate::error::RawssgError;
use crate::fs::FileSystem;
#[cfg(feature = "tera")]
use crate::site::context::{FeedContextBuilder, SitemapContextBuilder};
use crate::site::{Context, TemplateRenderer};
use crate::types::{PageContext, RawssgConfig};
#[cfg(feature = "tera")]
use crate::util::relative_prefix;
use crate::util::safe_path;
use std::io;
use std::path::{Path, PathBuf};

pub struct Site {
    pub(crate) config: RawssgConfig,
    pub(crate) pages: Vec<PageContext>,
    pub(crate) output_dir: PathBuf,
    #[cfg_attr(not(feature = "tera"), allow(dead_code))]
    pub(crate) base_url: String,
    pub(crate) fs: Box<dyn FileSystem>,
    pub(crate) renderer: Box<dyn TemplateRenderer>,
    pub(crate) content_dir: PathBuf,
    #[cfg(feature = "tera")]
    pub(crate) feed_context_builder: Option<Box<dyn FeedContextBuilder>>,
    #[cfg(feature = "tera")]
    pub(crate) sitemap_context_builder: Option<Box<dyn SitemapContextBuilder>>,
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

    pub(crate) fn generate_to(&self, output_base: &Path) -> Result<(), RawssgError> {
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
