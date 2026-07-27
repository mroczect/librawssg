---
title: API Reference
desc: Complete reference of all public types, traits, functions, and modules in librawssg v0.5.0
---

This document lists every public item exported by `librawssg`.  
Items gated behind Cargo features are clearly marked.

---

## `config` module

Configuration loading traits and implementations.

### `ConfigLoader` trait

```rust
pub trait ConfigLoader: Send + Sync {
    fn load(&self) -> Result<RawssgConfig, RawssgError>;
    fn load_or_default(&self) -> RawssgConfig {
        self.load().unwrap_or_else(|e| {
            tracing::error!("Failed to load config, using defaults: {}", e);
            RawssgConfig::default()
        })
    }
}
```

Implementors produce a `RawssgConfig`. `load_or_default()` logs the error and returns the default configuration if loading fails.

### `YamlConfigLoader<P>`

```rust
pub struct YamlConfigLoader<P: AsRef<Path> + Send + Sync> { /* path */ }
```

Reads a YAML file and deserialises it into a `RawssgConfig`.  
**Constructor**:

```rust
pub fn new(path: P) -> Self;
```

**Trait implementation**: `ConfigLoader`.

### `DefaultConfig`

```rust
pub struct DefaultConfig;
```

A `ConfigLoader` that always returns `RawssgConfig::default()` (which now includes a default `page` content type).

---

## `error` module

### `RawssgError` enum

```rust
pub enum RawssgError {
    Io(std::io::Error),
    Config(String),
    Frontmatter { path: PathBuf, source: Box<dyn Error + Send + Sync> },
    Template(String),
    PathTraversal(String),
    MissingConfig(String),
    Markdown(String),
    SiteGeneration(String),
    NotFound(String),
    Internal(String),
}
```

All errors implement `Display` and `Diagnostic` (via `miette`).  
Variants are used throughout the library for specific failure conditions.

---

## `frontmatter` module

### `parse_frontmatter_and_render`

```rust
pub fn parse_frontmatter_and_render(
    raw: &str,
    path: &Path,
    renderer: &dyn MarkdownRenderer,
) -> Result<(PageFrontMatter, String), RawssgError>;
```

Extracts YAML frontmatter from a string. Returns a `Frontmatter` error if the opening or closing `---` is missing, or if the YAML is invalid.  
Renders the remaining Markdown to HTML using the provided renderer. Returns the frontmatter and the rendered HTML.

---

## `fs` module

### `FileSystem` trait

```rust
pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    fn copy_file(&self, from: &Path, to: &Path) -> io::Result<u64>;
    fn walk_dir(&self, root: &Path) -> io::Result<Vec<PathBuf>>;
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;   // new in v0.5.0
}
```

Abstraction over all filesystem operations. Implement this trait to support in‑memory files, network storage, or any custom backend.  
The `rename` method is used by `Site::generate` for atomic output; a cross‑device fallback is triggered if `rename` returns `CrossesDevices`.

### `RealFs`

```rust
pub struct RealFs;
```

Implements `FileSystem` by delegating to `std::fs` and `walkdir`. All methods are instrumented with `tracing`.

---

## `markdown` module

### `MarkdownRenderer` trait

```rust
pub trait MarkdownRenderer: Send + Sync {
    fn render(&self, markdown: &str) -> String;
}
```

Converts Markdown text to HTML.

### `PulldownMarkdown` [![feature: pulldown](https://img.shields.io/badge/feature-pulldown-blue)]

```rust
pub struct PulldownMarkdown;
```

Available with the `pulldown` feature.  
Implements `MarkdownRenderer` using `pulldown‑cmark` with tables, strikethrough, and tasklists enabled.

---

## `serve` module [![feature: serve](https://img.shields.io/badge/feature-serve-blue)]

### `start_dev_server`

```rust
pub fn start_dev_server(output_dir: &Path, port: u16) -> Result<(), RawssgError>;
```

Starts a simple HTTP server that serves static files from `output_dir`.  
Requests are handled in separate threads. MIME types are detected from file extensions (including `text/plain` for `.txt`, `.md`, `.yaml`, `.yml`, `.log`). Uses `safe_path` to prevent directory traversal.

### `watch_dirs`

```rust
pub fn watch_dirs<F>(dirs: &[PathBuf], on_change: F) -> notify::Result<Box<dyn Watcher + Send>>
where
    F: Fn() + Send + 'static;
```

Watches one or more directories for changes (modify, create, remove) and calls `on_change` on each event. Returns a watcher that must be kept alive.

---

## `site` module

### `TemplateRenderer` trait

```rust
pub trait TemplateRenderer: Send + Sync {
    fn render(&self, template_name: &str, context: &dyn Context) -> Result<String, RawssgError>;
}
```

Renders a named template with a given context. The context is engine‑specific and accessed via the `Context` trait.

### `Context` trait

```rust
pub trait Context: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any;
}
```

Enables type‑erased access to the underlying template context. Engine implementations must implement this trait for their context type.

### `TeraRenderer` [![feature: tera](https://img.shields.io/badge/feature-tera-blue)]

```rust
pub struct TeraRenderer { /* contains a tera::Tera */ }
```

Available with the `tera` feature.  
**Constructor**:

```rust
pub fn new() -> Self;
```

**Methods**:

```rust
pub fn add_raw_template(&mut self, name: &str, content: &str) -> Result<(), RawssgError>;
```

Adds a template to the engine. Call this before passing the renderer to `SiteBuilder`.  
`TeraRenderer` implements `TemplateRenderer` and `Default`.

### `ContentHandler` trait

```rust
pub trait ContentHandler: Send + Sync {
    fn can_handle(&self, relative_path: &Path, original_path: &Path) -> bool;
    fn process(
        &self,
        fs: &dyn FileSystem,
        md_renderer: &dyn MarkdownRenderer,
        file_path: &Path,
        content_dir: &Path,
    ) -> Result<Option<PageContext>, RawssgError>;
}
```

Processes a file into a `PageContext`. Return `None` to skip the file (e.g., drafts).

### `MarkdownPageHandler`

```rust
pub struct MarkdownPageHandler;
```

Handles `.md` files by parsing frontmatter and rendering Markdown. Implements `ContentHandler`.

### `StaticFileHandler`

```rust
pub struct StaticFileHandler;
```

A catch‑all handler that always returns `None`. It is included by default to prevent warnings for non‑Markdown files (which are copied as assets separately). Implements `ContentHandler`.

### `build_page_context`

```rust
pub fn build_page_context(
    fs: &dyn FileSystem,
    markdown_renderer: &dyn MarkdownRenderer,
    file_path: &Path,
    content_dir: &Path,
) -> Result<Option<PageContext>, RawssgError>;
```

Validates the path, reads the file, extracts frontmatter, renders Markdown, and returns a `PageContext`. Drafts are skipped automatically.  
This function is used internally but is public for custom handler implementations.

---

### `SiteBuilder`

```rust
pub struct SiteBuilder { /* private fields */ }
```

The entry point for constructing a site.

**Location**: `librawssg::site::builders::site_builder` (re‑exported as `librawssg::SiteBuilder`).

**Methods**:

| Method                                                                                         | Description                                                                                                         |
| ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `pub fn new() -> Self`                                                                         | Creates a new builder with default values and `RealFs`.                                                             |
| `pub fn config(self, config: RawssgConfig) -> Self`                                            | Sets the entire configuration.                                                                                      |
| `pub fn load_config<P: AsRef<Path> + Send + Sync>(self, path: P) -> Result<Self, RawssgError>` | Loads configuration from a YAML file.                                                                               |
| `pub fn content_dir(self, dir: impl Into<PathBuf>) -> Self`                                    | Sets the content directory (default `"content"`).                                                                   |
| `pub fn output_dir(self, dir: impl Into<PathBuf>) -> Self`                                     | Sets the output directory (default `"dist"`).                                                                       |
| `pub fn with_fs(self, fs: Box<dyn FileSystem>) -> Self`                                        | Sets the filesystem backend.                                                                                        |
| `pub fn with_markdown_renderer(self, md: Box<dyn MarkdownRenderer>) -> Self`                   | Sets the Markdown renderer (**required**).                                                                          |
| `pub fn with_template_renderer(self, tr: Box<dyn TemplateRenderer>) -> Self`                   | Sets the template renderer (**required**).                                                                          |
| `pub fn add_handler(self, handler: Box<dyn ContentHandler>) -> Self`                           | Adds a content handler (defaults: `MarkdownPageHandler`, `StaticFileHandler`).                                      |
| `pub fn with_feed_context_builder(self, b: Box<dyn FeedContextBuilder>) -> Self`               | Sets the feed context builder. **Only required if `generators.rss.enabled` is `true`.** [![feature: tera](https://img.shields.io/badge/feature-tera-blue)] |
| `pub fn with_sitemap_context_builder(self, b: Box<dyn SitemapContextBuilder>) -> Self`         | Sets the sitemap context builder. **Only required if `generators.sitemap.enabled` is `true`.** [![feature: tera](https://img.shields.io/badge/feature-tera-blue)] |
| `pub fn build(self) -> Result<Site, RawssgError>`                                              | Validates config, canonicalizes directories, processes all content, and returns a ready‑to‑generate `Site`.          |

### `Site`

```rust
pub struct Site { /* private fields */ }
```

A fully processed site ready for output generation.  
**Location**: `librawssg::site::builders::site` (re‑exported as `librawssg::Site`).

**Methods**:

| Method                                             | Description                                                                                                                                                                                                                   |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pub fn pages(&self) -> &[PageContext]`            | Returns all pages (including list pages).                                                                                                                                                                                     |
| `pub fn generate(self) -> Result<(), RawssgError>` | Writes HTML files, copies static assets and non‑Markdown files, optionally generates RSS and sitemap. Uses atomic output via `FileSystem::rename` with a recursive copy fallback if the rename fails due to `CrossesDevices`. |

---

### `generate_feed` [![feature: tera](https://img.shields.io/badge/feature-tera-blue)]

```rust
pub fn generate_feed(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    posts: &[&PageContext],
    base_url: &str,
    context_builder: &dyn FeedContextBuilder,
) -> Result<String, RawssgError>;
```

Renders an RSS feed from a list of blog posts using the configured RSS template and the provided context builder.

### `generate_sitemap` [![feature: tera](https://img.shields.io/badge/feature-tera-blue)]

```rust
pub fn generate_sitemap(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    pages: &[PageContext],
    base_url: &str,
    context_builder: &dyn SitemapContextBuilder,
) -> Result<String, RawssgError>;
```

Renders a sitemap XML from all pages using the configured sitemap template and the provided context builder.

### Context builders (in `site::context`) [![feature: tera](https://img.shields.io/badge/feature-tera-blue)]

```rust
pub trait FeedContextBuilder: Send + Sync {
    fn build_feed_context(&self, config: &RawssgConfig, posts: &[&PageContext], base_url: &str)
        -> Result<Box<dyn Context>, RawssgError>;
}

pub trait SitemapContextBuilder: Send + Sync {
    fn build_sitemap_context(&self, config: &RawssgConfig, pages: &[PageContext], base_url: &str)
        -> Result<Box<dyn Context>, RawssgError>;
}
```

Default Tera implementations (`TeraFeedContextBuilder`, `TeraSitemapContextBuilder`) insert `site`, `posts`/`pages`, and `base_url`. Custom builders can inject additional variables.

---

## `types` module

### `RawssgConfig`

```rust
pub struct RawssgConfig {
    pub site: GlobalConfig,
    pub build: BuildConfig,
    pub content_types: Vec<ContentTypeDef>,
    pub generators: GeneratorsConfig,
}
```

Top‑level configuration.  
**Default**: includes a `page` content type (`**/*.md` → `base.html`).  
**Method**:

```rust
pub fn validate(&self) -> Result<(), RawssgError>;
```

Validates that `site_name` is non‑empty, `content_types` is non‑empty, each content type has a valid glob pattern and template, and required generator fields are present when enabled.

### `GlobalConfig`

```rust
pub struct GlobalConfig {
    pub navbar: Vec<NavItem>,
    pub sidebar: Vec<NavItem>,
    pub site_name: String,            // default "rawssg"
    pub description: Option<String>,
    pub language: Option<String>,     // default Some("en")
    pub base_url: Option<String>,
    pub author: Option<String>,
    pub repo_url: Option<String>,
    pub license: Option<String>,
}
```

Site‑wide metadata available in templates as `site.<field>`.

### `BuildConfig`

```rust
pub struct BuildConfig {
    pub content_dir: String,    // default "content"
    pub output_dir: String,     // default "dist"
    pub templates_dir: String,  // default "templates"
    pub static_dir: String,     // default "static"
}
```

### `ContentTypeDef`

```rust
pub struct ContentTypeDef {
    pub name: String,
    pub pattern: String,
    pub template: String,
    pub list_template: Option<String>,
    pub list_enabled: bool,
}
```

Defines a content type. `pattern` is a glob relative to `content_dir`. If `list_enabled` is `true`, a list page (`{name}/index.html`) is generated using `list_template`.

### `GeneratorsConfig`

```rust
pub struct GeneratorsConfig {
    pub rss: GeneratorDef,
    pub sitemap: GeneratorDef,
}
```

### `GeneratorDef`

```rust
pub struct GeneratorDef {
    pub enabled: bool,      // default false (changed in v0.5.0)
    pub path: String,
    pub template: String,
}
```

Configuration for RSS and sitemap generators. Both default to **disabled**. When enabled, `path` and `template` must be provided.

### `NavItem`

```rust
pub struct NavItem {
    pub label: String,
    pub url: String,
}
```

Used in `navbar` and `sidebar`.

### `PageFrontMatter`

```rust
pub struct PageFrontMatter {
    pub title: String,
    pub desc: String,
    pub author: Option<String>,
    pub repo_url: Option<String>,
    pub license: Option<String>,
    pub date: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub draft: bool,
}
```

Parsed from the YAML frontmatter block. `title` and `desc` are required; everything else is optional.

### `PageContext`

```rust
pub struct PageContext {
    pub frontmatter: PageFrontMatter,
    pub content_html: String,
    pub url: String,
    pub file_path: String,
    pub depth: usize,
    pub pub_date: Option<String>,
    pub content_type: String,
    pub is_list: bool,
    pub list_items: Option<Vec<PageContext>>,
}
```

Fully processed representation of a page. `pub_date` is an RFC 2822 string if `date` was present in the frontmatter. `list_items` is populated only for list pages.

---

## `util` module

### `safe_path`

```rust
pub fn safe_path(
    fs: &dyn FileSystem,
    base: &Path,
    candidate: &Path,
) -> Result<PathBuf, RawssgError>;
```

Validates that `candidate` is inside `base`.  
- If the candidate exists, it canonicalises the path and checks the prefix.  
- If the candidate does not exist (used for output files), it canonicalises the parent directory and verifies it is still within `base`.  
Prevents directory traversal and symlink escapes. Returns the safe canonical path.

### `slugify`

```rust
pub fn slugify(title: &str) -> String;
```

Converts a string to a URL‑friendly slug (lowercase, only alphanumerics and hyphens).

### `relative_prefix`

```rust
pub fn relative_prefix(depth: usize) -> String;
```

Returns a relative path prefix based on directory depth: `"./"` for depth 0, `"../"` for depth 1, etc.

### `match_pattern`

```rust
pub fn match_pattern(pattern: &str, path: &Path) -> bool;
```

Matches a path against a glob pattern with `*` (single segment) and `**` (any number of segments).  
The algorithm correctly handles patterns like `blog/**/*.html` (will not match `.md`). Used internally for content type matching.