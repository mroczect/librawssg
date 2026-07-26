# librawssg · [![GitHub tag](https://img.shields.io/github/v/tag/mroczect/librawssg?label=version)](https://github.com/mroczect/librawssg/tags) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**librawssg** is the engine‑agnostic, safety‑first kernel for building static site generators in Rust.  
It gives you all the primitives you need: filesystem abstraction, frontmatter parsing, Markdown rendering,
template rendering, content processing pipelines, feed & sitemap generation, and a secure development server.

The library does **not** include a CLI – you write your own `main.rs` and compose the parts you need.
Optional built‑in implementations for **Tera** and **pulldown‑cmark** are available behind feature flags.

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [API Reference](#api-reference)
  - [Configuration](#configuration)
    - [YamlConfigLoader](#yamlconfigloader)
    - [ConfigLoader trait](#configloader-trait)
  - [Error Handling](#error-handling)
    - [RawssgError](#rawssgerror)
  - [Filesystem Abstraction](#filesystem-abstraction)
    - [FileSystem trait](#filesystem-trait)
    - [RealFs](#realfs)
  - [Markdown Rendering](#markdown-rendering)
    - [MarkdownRenderer trait](#markdownrenderer-trait)
    - [PulldownMarkdown](#pulldownmarkdown)
  - [Template Rendering](#template-rendering)
    - [TemplateRenderer trait](#templaterenderer-trait)
    - [Context trait](#context-trait)
    - [TeraRenderer](#terarenderer)
  - [Content Pipeline](#content-pipeline)
    - [ContentHandler trait](#contenthandler-trait)
    - [MarkdownPageHandler](#markdownpagehandler)
    - [StaticFileHandler](#staticfilehandler)
    - [build_page_context](#build_page_context)
  - [Site Builder](#site-builder)
    - [SiteBuilder](#sitebuilder)
    - [Site](#site)
  - [Feed & Sitemap](#feed--sitemap)
    - [generate_feed](#generate_feed)
    - [generate_sitemap](#generate_sitemap)
    - [Context builders](#context-builders)
  - [Utility Functions](#utility-functions)
    - [safe_path](#safe_path)
    - [normalize_path](#normalize_path)
    - [slugify](#slugify)
    - [relative_prefix](#relative_prefix)
    - [match_pattern](#match_pattern)
  - [Type Reference](#type-reference)
    - [RawssgConfig](#rawssgconfig)
    - [GlobalConfig](#globalconfig)
    - [BuildConfig](#buildconfig)
    - [ContentTypeDef](#contenttypedef)
    - [GeneratorsConfig](#generatorsconfig)
    - [GeneratorDef](#generatordef)
    - [NavItem](#navitem)
    - [PageFrontMatter](#pagefrontmatter)
    - [PageContext](#pagecontext)
  - [Dev Server & Watcher (serve feature)](#dev-server--watcher-serve-feature)
- [Feature Flags](#feature-flags)
- [Security](#security)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

---

## Installation

### Method 1: `cargo add` (Git dependency – recommended)

If you have `cargo-edit` installed (`cargo install cargo-edit`), the fastest way is:

```bash
cargo add --git https://github.com/mroczect/librawssg.git --tag v0.4.0 librawssg
```

To also enable the built‑in Tera and pulldown‑cmark implementations (most common):

```bash
cargo add --git https://github.com/mroczect/librawssg.git --tag v0.4.0 librawssg --features tera,pulldown
```

This will add the dependency to your `Cargo.toml` automatically.

---

### Method 2: Manual `Cargo.toml` entry

Add this to your `Cargo.toml`:

```toml
[dependencies]
librawssg = { git = "https://github.com/mroczect/librawssg.git", tag = "v0.4.0" }
```

**Always pin a specific tag** to avoid breaking changes. To enable Tera and pulldown-cmark:

```toml
librawssg = { git = "https://github.com/mroczect/librawssg.git", tag = "v0.4.0", features = ["tera", "pulldown"] }
```

---

## Method 3: Path dependency (local development)

If you cloned the repository and want to hack on the library:

```bash
git clone https://github.com/mroczect/librawssg.git
cd librawssg
```

Then in your project’s `Cargo.toml`:

```toml
[dependencies]
librawssg = { path = "../librawssg", features = ["tera", "pulldown"] }
```

---

### Method 4: Full clone and build

```bash
git clone https://github.com/mroczect/librawssg.git
cd librawssg
cargo build --release
```

Then reference it as a path dependency (see Method 3).

---

## Quick Start

Create a new binary crate and add `librawssg` with the `tera` and `pulldown` features.

```rust
use librawssg::site::builder::SiteBuilder;
use librawssg::site::TeraRenderer;
use librawssg::markdown::PulldownMarkdown;
use librawssg::types::{RawssgConfig, ContentTypeDef};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare the template engine
    let mut tera = TeraRenderer::new();
    tera.add_raw_template("base.html", "<html><body>{{ page_content }}</body></html>")?;

    // 2. Choose a Markdown renderer
    let md = PulldownMarkdown;

    // 3. Build configuration (at least one content_type is required)
    let mut config = RawssgConfig::default();
    config.build.content_dir = "content".into();
    config.build.output_dir = "dist".into();
    config.content_types.push(ContentTypeDef {
        name: "page".into(),
        pattern: "**/*.md".into(),
        template: "base.html".into(),
        list_template: None,
        list_enabled: false,
    });

    // 4. Build the site
    let site = SiteBuilder::new()
        .config(config)
        .content_dir("content")
        .output_dir("dist")
        .with_template_renderer(Box::new(tera))
        .with_markdown_renderer(Box::new(md))
        .build()?;

    // 5. Generate static files
    site.generate()?;
    Ok(())
}
```

Place your Markdown files in `content/`, run the project, and the generated HTML will appear in `dist/`.

---

## Architecture

```
src/
  config/         Configuration loading (YAML, defaults)
  error.rs        Typed error enum (miette + thiserror)
  frontmatter.rs  YAML frontmatter extraction and Markdown rendering
  fs/             FileSystem trait and real filesystem implementation
  markdown.rs     MarkdownRenderer trait (+ optional pulldown-cmark)
  serve/          Dev server and file watcher (behind "serve" feature)
  site/           Site builder, content handlers, feed & sitemap generators
  types.rs        All configuration and page context types
  util.rs         Path safety, slugify, glob matching
```

All public types and traits are re‑exported from the crate root.

---

## API Reference

### Configuration

#### `YamlConfigLoader`

```rust
pub struct YamlConfigLoader<P: AsRef<Path> + Send + Sync> { ... }

impl<P: AsRef<Path> + Send + Sync> YamlConfigLoader<P> {
    pub fn new(path: P) -> Self;
}

impl<P: AsRef<Path> + Send + Sync> ConfigLoader for YamlConfigLoader<P> {
    fn load(&self) -> Result<RawssgConfig, RawssgError>;
}
```

Loads configuration from a YAML file. Returns a `Config` error on I/O failure or invalid YAML.

#### `ConfigLoader` trait

```rust
pub trait ConfigLoader: Send + Sync {
    fn load(&self) -> Result<RawssgConfig, RawssgError>;
    fn load_or_default(&self) -> RawssgConfig { ... }
}
```

Trait for loading configuration. Two implementations are provided:
- `YamlConfigLoader` – reads a YAML file.
- `DefaultConfig` – always returns `RawssgConfig::default()`.

---

### Error Handling

#### `RawssgError`

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

All variants implement `std::error::Error`, `Display`, and `miette::Diagnostic`. Rich diagnostic messages
are provided for common failures (e.g., malformed frontmatter).

---

### Filesystem Abstraction

#### `FileSystem` trait

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
}
```

Abstracts all file I/O. Implement this to support in‑memory filesystems, remote storage, etc.

#### `RealFs`

```rust
pub struct RealFs;

impl FileSystem for RealFs { ... }
```

Default implementation that delegates to `std::fs` and `walkdir`. All methods are instrumented with `tracing`.

---

### Markdown Rendering

#### `MarkdownRenderer` trait

```rust
pub trait MarkdownRenderer: Send + Sync {
    fn render(&self, markdown: &str) -> String;
}
```

Convert Markdown to HTML. Implement this for any parser.

#### `PulldownMarkdown`

Available behind the **`pulldown`** feature flag.

```rust
#[cfg(feature = "pulldown")]
pub struct PulldownMarkdown;

#[cfg(feature = "pulldown")]
impl MarkdownRenderer for PulldownMarkdown {
    fn render(&self, md: &str) -> String;
}
```

Uses `pulldown-cmark` with tables, strikethrough, and task lists enabled.

---

### Template Rendering

#### `TemplateRenderer` trait

```rust
pub trait TemplateRenderer: Send + Sync {
    fn render(&self, template_name: &str, context: &dyn Context) -> Result<String, RawssgError>;
}
```

Renders a named template with a given context. The context is engine‑specific.

#### `Context` trait

```rust
pub trait Context: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
```

Type‑erased access to the underlying context object. Engine implementations must implement this trait for their context type.

#### `TeraRenderer`

Available behind the **`tera`** feature flag.

```rust
#[cfg(feature = "tera")]
pub struct TeraRenderer { ... }

#[cfg(feature = "tera")]
impl TeraRenderer {
    pub fn new() -> Self;
    pub fn add_raw_template(&mut self, name: &str, content: &str) -> Result<(), RawssgError>;
}

#[cfg(feature = "tera")]
impl TemplateRenderer for TeraRenderer { ... }
```

A built‑in Tera engine. Call `add_raw_template` to register templates before passing the renderer to `SiteBuilder`.

---

### Content Pipeline

#### `ContentHandler` trait

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

Process a file into a `PageContext`. Return `None` to skip the file (e.g., drafts).  
Two built‑in handlers are provided.

#### `MarkdownPageHandler`

```rust
pub struct MarkdownPageHandler;

impl ContentHandler for MarkdownPageHandler {
    fn can_handle(&self, _rel: &Path, orig: &Path) -> bool { ... }
    fn process(...) -> Result<Option<PageContext>, RawssgError> { ... }
}
```

Handles `.md` files. Extracts frontmatter, renders Markdown, and returns a `PageContext`.

#### `StaticFileHandler`

```rust
pub struct StaticFileHandler;

impl ContentHandler for StaticFileHandler { ... }
```

A catch‑all that always returns `None`. Included by default so that non‑Markdown files
don't cause warnings – they are copied as static assets later.

#### `build_page_context`

```rust
pub fn build_page_context(
    fs: &dyn FileSystem,
    markdown_renderer: &dyn MarkdownRenderer,
    file_path: &Path,
    content_dir: &Path,
) -> Result<Option<PageContext>, RawssgError>;
```

Low‑level function that validates the path, reads the file, parses frontmatter,
renders Markdown, and constructs a `PageContext`. Drafts are automatically skipped.

---

### Site Builder

#### `SiteBuilder`

```rust
pub struct SiteBuilder { ... }

impl SiteBuilder {
    pub fn new() -> Self;
    pub fn config(self, config: RawssgConfig) -> Self;
    pub fn load_config<P: AsRef<Path> + Send + Sync>(self, path: P) -> Result<Self, RawssgError>;
    pub fn content_dir(self, dir: impl Into<PathBuf>) -> Self;
    pub fn output_dir(self, dir: impl Into<PathBuf>) -> Self;
    pub fn with_fs(self, fs: Box<dyn FileSystem>) -> Self;
    pub fn with_markdown_renderer(self, md: Box<dyn MarkdownRenderer>) -> Self;
    pub fn with_template_renderer(self, tr: Box<dyn TemplateRenderer>) -> Self;
    pub fn add_handler(self, handler: Box<dyn ContentHandler>) -> Self;
    pub fn build(self) -> Result<Site, RawssgError>;
}
```

The entry point for constructing a site. `build()` validates the configuration, processes all
content files using the registered handlers, sorts blog posts by date, generates list pages
(if enabled), and returns a `Site` that is ready to be generated.

#### `Site`

```rust
pub struct Site { ... }

impl Site {
    pub fn pages(&self) -> &[PageContext];
    pub fn generate(self) -> Result<(), RawssgError>;
}
```

The compiled site representation. `generate()`:
- Writes all HTML pages to the output directory.
- Copies static assets from the configured static directory.
- Copies non‑Markdown files from the content directory (images, CSS, etc.).
- Optionally generates RSS feed and sitemap (when the `tera` feature is enabled).
- Uses an atomic write strategy (writes to a temporary directory, then renames).

---

### Feed & Sitemap

All feed and sitemap functions are available only with the **`tera`** feature.

#### `generate_feed`

```rust
pub fn generate_feed(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    posts: &[&PageContext],
    base_url: &str,
    context_builder: &dyn FeedContextBuilder,
) -> Result<String, RawssgError>;
```

Generates an RSS feed string using the configured RSS template and context builder.

#### `generate_sitemap`

```rust
pub fn generate_sitemap(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    pages: &[PageContext],
    base_url: &str,
    context_builder: &dyn SitemapContextBuilder,
) -> Result<String, RawssgError>;
```

Generates a sitemap XML string.

#### Context builders

```rust
pub trait FeedContextBuilder: Send + Sync {
    fn build_feed_context(
        &self, config: &RawssgConfig, posts: &[&PageContext], base_url: &str
    ) -> Result<Box<dyn Context>, RawssgError>;
}

pub trait SitemapContextBuilder: Send + Sync {
    fn build_sitemap_context(
        &self, config: &RawssgConfig, pages: &[PageContext], base_url: &str
    ) -> Result<Box<dyn Context>, RawssgError>;
}
```

Default Tera implementations are provided:
- `TeraFeedContextBuilder` – inserts `site`, `posts`, `base_url`.
- `TeraSitemapContextBuilder` – inserts `site`, `pages`, `base_url`.

You can implement custom builders to inject additional context variables.

---

### Utility Functions

#### `safe_path`

```rust
pub fn safe_path(
    fs: &dyn FileSystem,
    base: &Path,
    candidate: &Path,
) -> Result<PathBuf, RawssgError>;
```

Validates that `candidate` resolves inside `base`. It canonicalises both paths, resolves `..` and `.`,
and returns the safe canonical path. Emits a `PathTraversal` error if the path escapes `base`.

#### `normalize_path`

```rust
fn normalize_path(path: &Path) -> PathBuf;
```

Pure path normalisation that removes `.` and resolves `..` without touching the filesystem.

#### `slugify`

```rust
pub fn slugify(title: &str) -> String;
```

Converts a string to a URL‑friendly slug: lowercase, alphanumerics and hyphens only.

#### `relative_prefix`

```rust
pub fn relative_prefix(depth: usize) -> String;
```

Returns a relative path prefix for the given directory depth (`"./"` for depth 0, `"../"` for depth 1, etc.).

#### `match_pattern`

```rust
pub fn match_pattern(pattern: &str, path: &Path) -> bool;
```

Matches a path against a glob‑like pattern. Supports `*` (single segment wildcard) and `**` (multi‑segment wildcard).

---

### Type Reference

#### `RawssgConfig`

```rust
pub struct RawssgConfig {
    pub site: GlobalConfig,
    pub build: BuildConfig,
    pub content_types: Vec<ContentTypeDef>,
    pub generators: GeneratorsConfig,
}

impl RawssgConfig {
    pub fn validate(&self) -> Result<(), RawssgError>;
}
```

Top‑level configuration. `validate()` checks that `site_name` is not empty, at least one content type is defined,
all patterns are valid globs, and required generator fields are present when enabled.

#### `GlobalConfig`

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

Site‑wide metadata available in templates.

#### `BuildConfig`

```rust
pub struct BuildConfig {
    pub content_dir: String,    // default "content"
    pub output_dir: String,     // default "dist"
    pub templates_dir: String,  // default "templates"
    pub static_dir: String,     // default "static"
}
```

#### `ContentTypeDef`

```rust
pub struct ContentTypeDef {
    pub name: String,
    pub pattern: String,          // glob pattern
    pub template: String,
    pub list_template: Option<String>,
    pub list_enabled: bool,
}
```

Defines a content type. If `list_enabled` is `true`, a list page (e.g., `blog/index.html`)
is automatically created from all pages of that type.

#### `GeneratorsConfig`

```rust
pub struct GeneratorsConfig {
    pub rss: GeneratorDef,
    pub sitemap: GeneratorDef,
}
```

#### `GeneratorDef`

```rust
pub struct GeneratorDef {
    pub enabled: bool,      // default true
    pub path: String,
    pub template: String,
}
```

Configuration for RSS and sitemap generation. Both default to enabled, but require explicit
`path` and `template` when active.

#### `NavItem`

```rust
pub struct NavItem {
    pub label: String,
    pub url: String,
}
```

Used in `navbar` and `sidebar` of `GlobalConfig`.

#### `PageFrontMatter`

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

Parsed from the YAML frontmatter block. Only `title` and `desc` are required.

#### `PageContext`

```rust
pub struct PageContext {
    pub frontmatter: PageFrontMatter,
    pub content_html: String,
    pub url: String,
    pub file_path: String,
    pub depth: usize,
    pub pub_date: Option<String>,   // RFC 2822 if date present
    pub content_type: String,
    pub is_list: bool,
    pub list_items: Option<Vec<PageContext>>,
}
```

Fully processed representation of a page.

---

### Dev Server & Watcher (serve feature)

Enable the **`serve`** feature to get a built‑in development server with live reload.

```rust
// Starting the dev server
use librawssg::serve::start_dev_server;

start_dev_server(Path::new("dist"), 8080)?;
```

The server:
- Serves files from the output directory with correct MIME types.
- Returns 404 for missing files and 500 for internal errors.
- Handles each request in a separate thread.

A file watcher is also available:

```rust
use librawssg::serve::watcher::watch_dirs;

let _watcher = watch_dirs(&[PathBuf::from("content"), PathBuf::from("templates")], || {
    println!("Change detected – rebuild!");
})?;
```

It uses the `notify` crate and triggers the closure on `Modify`, `Create`, or `Remove` events.

---

## Feature Flags

| Feature    | Description                                                                 |
|------------|-----------------------------------------------------------------------------|
| `tera`     | Enables built‑in `TeraRenderer` and context builders (depends on `tera`)   |
| `pulldown` | Enables built‑in `PulldownMarkdown` (depends on `pulldown-cmark`)          |
| `serve`    | Enables the dev server and file watcher (depends on `tiny_http`, `notify`) |

All features are disabled by default. Enable only what you need.

---

## Security

- **Path confinement**: `safe_path` prevents directory traversal by canonicalizing and normalizing all paths.
- **Atomic output**: `Site::generate` writes to a temp directory and renames on success, leaving the previous output intact on failure.
- **Strict configuration**: unknown YAML keys are rejected; content type patterns are validated via `glob::Pattern`.
- **No symlink following**: the library works exclusively with canonical paths and does not follow symbolic links.

---

## Testing

Run the full test suite with:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features tera,pulldown
cargo test --features serve
```

The test suite includes a comprehensive mock filesystem (`MockFs`) and mock renderers
(in `tests/common/mod.rs`) to help you write your own integration tests.

---

## Contributing

Contributions are welcome! Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines on
code style, commit messages, and the pull request process.

---

## License

This project is licensed under the [MIT License](LICENSE).