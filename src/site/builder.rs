use crate::config::ConfigLoader;
use crate::error::RawssgError;
use crate::fs::FileSystem;
use crate::markdown::MarkdownRenderer;
use crate::site::page::build_page_context;
use crate::site::TemplateRenderer;
use crate::types::{GlobalConfig, PageContext, PageFrontMatter};
use crate::util::{relative_prefix, safe_path};
use std::path::{Path, PathBuf};
use tera::Context;

pub struct Site<T: TemplateRenderer> {
    renderer: T,
    config: GlobalConfig,
    pages: Vec<PageContext>,
    output_dir: PathBuf,
    base_url: String,
    fs: Box<dyn FileSystem>,
}

impl<T: TemplateRenderer> Site<T> {
    #[tracing::instrument(skip(self))]
    pub fn generate(&self) -> Result<(), RawssgError> {
        if self.fs.exists(&self.output_dir) {
            self.fs.remove_dir_all(&self.output_dir)?;
        }
        self.fs.create_dir_all(&self.output_dir)?;

        for page in &self.pages {
            let mut ctx = Context::new();
            self.populate_context(page, &mut ctx);
            let html = self.renderer.render("base.html", &ctx)?;
            let out_path = safe_path(&self.output_dir, &self.output_dir.join(&page.url))?;
            self.fs.write(&out_path, html.as_bytes())?;
        }

        let blog_posts: Vec<&PageContext> = self
            .pages
            .iter()
            .filter(|p| p.url.starts_with("blog/"))
            .collect();
        if !blog_posts.is_empty() {
            let rss = crate::site::feed::generate_feed(
                &self.renderer,
                &self.config,
                &blog_posts,
                &self.base_url,
            )?;
            self.fs
                .write(&self.output_dir.join("feed.xml"), rss.as_bytes())?;
        }

        let sitemap = crate::site::sitemap::generate_sitemap(
            &self.renderer,
            &self.pages,
            &self.base_url,
        )?;
        self.fs
            .write(&self.output_dir.join("sitemap.xml"), sitemap.as_bytes())?;

        Ok(())
    }

    fn populate_context(&self, page: &PageContext, ctx: &mut Context) {
        ctx.insert("title", &page.frontmatter.title);
        ctx.insert("desc", &page.frontmatter.desc);
        ctx.insert("content", &page.content_html);
        ctx.insert("base_path", &relative_prefix(page.depth));
        ctx.insert("navbar", &self.config.navbar);
        ctx.insert("sidebar", &self.config.sidebar);

        let author = page
            .frontmatter
            .author
            .as_deref()
            .or(self.config.author.as_deref())
            .unwrap_or("");
        let repo = page
            .frontmatter
            .repo_url
            .as_deref()
            .or(self.config.repo_url.as_deref())
            .unwrap_or("");
        let license = page
            .frontmatter
            .license
            .as_deref()
            .or(self.config.license.as_deref())
            .unwrap_or("");

        ctx.insert("author", author);
        ctx.insert("repo_url", repo);
        ctx.insert("license", license);
        ctx.insert("site_name", &self.config.site_name);
        ctx.insert(
            "description",
            &self.config.description.as_deref().unwrap_or(""),
        );
        ctx.insert(
            "language",
            &self.config.language.as_deref().unwrap_or("en"),
        );
    }
}

pub struct SiteBuilder<
    F: FileSystem + 'static,
    C: ConfigLoader + 'static,
    M: MarkdownRenderer + 'static,
    T: TemplateRenderer + 'static,
> {
    fs: F,
    config_loader: C,
    markdown: M,
    template_renderer: T,
    content_dir: PathBuf,
    output_dir: PathBuf,
}

impl<F: FileSystem + 'static, C: ConfigLoader + 'static, M: MarkdownRenderer + 'static, T: TemplateRenderer + 'static>
    SiteBuilder<F, C, M, T>
{
    pub fn new(fs: F, config_loader: C, markdown: M, template_renderer: T) -> Self {
        Self {
            fs,
            config_loader,
            markdown,
            template_renderer,
            content_dir: PathBuf::from("content"),
            output_dir: PathBuf::from("dist"),
        }
    }

    pub fn content_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.content_dir = dir.into();
        self
    }

    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    #[tracing::instrument(skip(self))]
    pub fn build(self) -> Result<Site<T>, RawssgError> {
        let config = self.config_loader.load_or_default();
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:3000".to_string());

        let content_files = self.fs.walk_dir(&self.content_dir)?;
        let mut pages = Vec::new();

        for file_path in &content_files {
            if file_path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            match build_page_context(
                &self.fs,
                &self.markdown,
                file_path,
                &self.content_dir,
            ) {
                Ok(page) => pages.push(page),
                Err(e) => return Err(e),
            }
        }

        let mut blog_posts: Vec<PageContext> = pages
            .iter()
            .filter(|p| p.url.starts_with("blog/"))
            .cloned()
            .collect();
        blog_posts.sort_by(|a, b| {
            b.frontmatter
                .date
                .cmp(&a.frontmatter.date)
                .then_with(|| a.frontmatter.title.cmp(&b.frontmatter.title))
        });

        if !blog_posts.is_empty() && !pages.iter().any(|p| p.url == "blog/index.html") {
            let mut list_html = String::from("<ul>\n");
            for post in &blog_posts {
                list_html.push_str(&format!(
                    "<li><a href=\"{}\">{}</a> &mdash; {}</li>\n",
                    post.url, post.frontmatter.title, post.frontmatter.desc
                ));
            }
            list_html.push_str("</ul>");

            let blog_index = PageContext {
                frontmatter: PageFrontMatter {
                    title: "Blog".into(),
                    desc: "All blog posts".into(),
                    author: None,
                    repo_url: None,
                    license: None,
                    date: None,
                    tags: vec![],
                    draft: false,
                },
                content_html: list_html,
                url: "blog/index.html".to_string(),
                file_path: "[generated]".into(),
                depth: 1,
                pub_date: None,
            };
            pages.push(blog_index);
        }

        Ok(Site {
            renderer: self.template_renderer,
            config,
            pages,
            output_dir: self.output_dir,
            base_url,
            fs: Box::new(self.fs),
        })
    }
}
