# librawssg &middot; [![GitHub tag](https://img.shields.io/github/v/tag/mroczect/librawssg?label=version)](https://github.com/mroczect/librawssg/tags) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**librawssg** is the engine-agnostic, safety-first kernel for static site generation in Rust.  
It provides all the primitives needed to build a custom static site generator: filesystem abstraction, frontmatter parsing, Markdown rendering, template rendering, content processing pipelines, feed/sitemap generation, and a secure dev server.

The library itself does **not** ship with a CLI – you write your own `main.rs` and compose the parts you need. Optional built-in implementations for **Tera** and **pulldown-cmark** are available behind feature flags.

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
  - [Feed and Sitemap](#feed-and-sitemap)
    - [generate_feed](#generate_feed)
    - [generate_sitemap](#generate_sitemap)
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
- [Feature Flags](#feature-flags)
- [Security](#security)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

---

## Installation

The library is **not yet published on [crates.io](https://crates.io)**. To use it, add the Git repository as a dependency in your `Cargo.toml`:

```toml
[dependencies]
librawssg = { git = "https://github.com/mroczect/librawssg.git", tag = "v0.2.0" }

# Optional: enable built-in Tera and pulldown-cmark support
# librawssg = { git = "...", tag = "v0.2.0", features = ["tera", "pulldown"] }
```

Always pin to a specific tag to avoid unexpected breaking changes.

---

## Quick Start

Create a new Rust binary project and add `librawssg` as a dependency. Enable the `tera` and `pulldown` features for this example.

```rust
use librawssg::site::builder::SiteBuilder;
use librawssg::site::TeraRenderer;
use librawssg::markdown::PulldownMarkdown;
use librawssg::types::{RawssgConfig, ContentTypeDef};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare template engine
    let mut tera = TeraRenderer::new();
    tera.add_raw_template("base.html", "<html><body>{{ page_content }}</body></html>")?;

    // 2. Choose markdown renderer
    let md = PulldownMarkdown;

    // 3. Build configuration (at least one content_type required)
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

    // 4. Build site
    let site = SiteBuilder::new()
        .config(config)
        .content_dir("content")
        .output_dir("dist")
        .with_template_renderer(Box::new(tera))
        .with_markdown_renderer(Box::new(md))
        .build()?;

    // 5. Generate
    site.generate()?;
    Ok(())
}
```

Place your Markdown files in `content/` and run with `cargo run`. The generated HTML will appear in `dist/`.

---

## Architecture

The library is organized into several public modules:

```
src/
  config/         Configuration loading (YAML, default)
  error.rs        Typed error enum (miette + thiserror)
  frontmatter.rs  YAML frontmatter extraction
  fs/             FileSystem trait + real filesystem implementation
  markdown.rs     MarkdownRenderer trait (+ optional pulldown-cmark)
  serve/          Optional dev server with live reload
  site/           Site builder, content handlers, feed/sitemap generators
  types.rs        All configuration and page context types
  util.rs         Path safety, slugify, glob matching
```

All public types and traits are re-exported from the crate root for convenience.

---

## API Reference

### Configuration

#### YamlConfigLoader

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

#### ConfigLoader trait

```rust
pub trait ConfigLoader: Send + Sync {
    fn load(&self) -> Result<RawssgConfig, RawssgError>;
    fn load_or_default(&self) -> RawssgConfig { ... }
}
```

Trait for loading configuration. Implementations include `YamlConfigLoader` and `DefaultConfig`.

---

### Error Handling

#### RawssgError

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

All errors implement `Display` and `Diagnostic` (via `miette`). Use `RawssgError::Config` for configuration issues, `PathTraversal` for security violations, etc.

---

### Filesystem Abstraction

#### FileSystem trait

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

Abstracts all filesystem operations. Implement this trait to support in-memory filesystems, remote storage, etc.

#### RealFs

```rust
pub struct RealFs;

impl FileSystem for RealFs { ... }
```

The default implementation that delegates to `std::fs` and `walkdir`. All methods are instrumented with `tracing`.

---

### Markdown Rendering

#### MarkdownRenderer trait

```rust
pub trait MarkdownRenderer: Send + Sync {
    fn render(&self, markdown: &str) -> String;
}
```

Convert Markdown to HTML. Implement this trait for any parser.

#### PulldownMarkdown

```rust
#[cfg(feature = "pulldown")]
pub struct PulldownMarkdown;

#[cfg(feature = "pulldown")]
impl MarkdownRenderer for PulldownMarkdown {
    fn render(&self, md: &str) -> String;
}
```

Built-in implementation using `pulldown-cmark` with tables, strikethrough, and task lists enabled.

---

### Template Rendering

#### TemplateRenderer trait

```rust
pub trait TemplateRenderer: Send + Sync {
    fn render(&self, template_name: &str, context: &dyn Context) -> Result<String, RawssgError>;
}
```

Renders a named template with a given context. The context is engine-specific and accessed via the `Context` trait.

#### Context trait

```rust
pub trait Context: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
```

Allows type-erased access to the underlying template context. Engine implementations must implement this trait for their context type.

#### TeraRenderer

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

Built-in Tera engine. Call `add_raw_template` before passing it to `SiteBuilder`.

---

### Content Pipeline

#### ContentHandler trait

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

#### MarkdownPageHandler

```rust
pub struct MarkdownPageHandler;

impl ContentHandler for MarkdownPageHandler {
    fn can_handle(&self, _rel: &Path, orig: &Path) -> bool { ... }
    fn process(...) -> Result<Option<PageContext>, RawssgError> { ... }
}
```

Handles `.md` files by parsing frontmatter and rendering Markdown.

#### StaticFileHandler

```rust
pub struct StaticFileHandler;

impl ContentHandler for StaticFileHandler { ... }
```

A catch-all handler that always returns `None`. It is included by default to prevent "no handler" warnings for non-Markdown files (which are copied as assets separately).

#### build_page_context

```rust
pub fn build_page_context(
    fs: &dyn FileSystem,
    markdown_renderer: &dyn MarkdownRenderer,
    file_path: &Path,
    content_dir: &Path,
) -> Result<Option<PageContext>, RawssgError>;
```

Validates the path, reads the file, parses frontmatter, renders Markdown, and returns a `PageContext`. Skips drafts automatically.

---

### Site Builder

#### SiteBuilder

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

The entry point for constructing a site. `build()` validates the configuration and processes all content files. Returns a `Site` ready for generation.

#### Site

```rust
pub struct Site { ... }

impl Site {
    pub fn pages(&self) -> &[PageContext];
    pub fn generate(self) -> Result<(), RawssgError>;
}
```

The compiled site representation. `generate()` writes all HTML files, copies static assets and non-Markdown files, and optionally generates RSS and sitemap files. Output is written atomically (temp directory + rename).

---

### Feed and Sitemap

#### generate_feed

```rust
#[cfg(feature = "tera")]
pub fn generate_feed(
    renderer: &dyn TemplateRenderer,
    config: &RawssgConfig,
    posts: &[&PageContext],
    base_url: &str,
) -> Result<String, RawssgError>;
```

Generates an RSS feed string using the configured RSS template. Only available with the `tera` feature.

#### generate_sitemap

```rust
#[cfg(feature = "tera")]
pub fn generate_sitemap(
    renderer: &dyn TemplateRenderer,
    pages: &[PageContext],
    base_url: &str,
) -> Result<String, RawssgError>;
```

Generates a sitemap XML string. Only available with the `tera` feature.

---

### Utility Functions

#### safe_path

```rust
pub fn safe_path(
    fs: &dyn FileSystem,
    base: &Path,
    candidate: &Path,
) -> Result<PathBuf, RawssgError>;
```

Validates that `candidate` resolves inside `base`. Canonicalizes paths, normalizes `..` and `.`, and checks for traversal. Returns the safe canonical path.

#### normalize_path

```rust
fn normalize_path(path: &Path) -> PathBuf;
```

Pure path normalization without touching the filesystem. Removes `.` and resolves `..` components.

#### slugify

```rust
pub fn slugify(title: &str) -> String;
```

Converts a string to a URL-friendly slug: lowercase, alphanumerics and hyphens only.

#### relative_prefix

```rust
pub fn relative_prefix(depth: usize) -> String;
```

Returns a relative path prefix (`./`, `../`, `../../`, etc.) based on the page depth in the directory tree.

#### match_pattern

```rust
pub fn match_pattern(pattern: &str, path: &Path) -> bool;
```

Matches a path against a glob-like pattern with support for `*` (single segment wildcard) and `**` (multi-segment wildcard).

---

### Type Reference

#### RawssgConfig

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

Top-level configuration. `validate()` checks that `site_name` is not empty, at least one content type is defined, and all generator templates/paths are set if enabled.

#### GlobalConfig

```rust
pub struct GlobalConfig {
    pub navbar: Vec<NavItem>,
    pub sidebar: Vec<NavItem>,
    pub site_name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub base_url: Option<String>,
    pub author: Option<String>,
    pub repo_url: Option<String>,
    pub license: Option<String>,
}
```

Site-wide metadata. Default `site_name` is `"rawssg"`.

#### BuildConfig

```rust
pub struct BuildConfig {
    pub content_dir: String,    // default "content"
    pub output_dir: String,     // default "dist"
    pub templates_dir: String,  // default "templates"
    pub static_dir: String,     // default "static"
}
```

Directory layout configuration.

#### ContentTypeDef

```rust
pub struct ContentTypeDef {
    pub name: String,
    pub pattern: String,          // glob pattern for matching files
    pub template: String,         // template name used for rendering
    pub list_template: Option<String>,
    pub list_enabled: bool,
}
```

Defines a content type. If `list_enabled` is `true`, a list page (e.g., `blog/index.html`) is automatically generated from all pages of this type.

#### GeneratorsConfig

```rust
pub struct GeneratorsConfig {
    pub rss: GeneratorDef,
    pub sitemap: GeneratorDef,
}
```

#### GeneratorDef

```rust
pub struct GeneratorDef {
    pub enabled: bool,      // default true
    pub path: String,
    pub template: String,
}
```

Configuration for RSS and sitemap generation. Both default to enabled but require explicit template and path if activated.

#### NavItem

```rust
pub struct NavItem {
    pub label: String,
    pub url: String,
}
```

Used in `navbar` and `sidebar` of `GlobalConfig`.

#### PageFrontMatter

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

Parsed from the YAML frontmatter block. All fields except `title` and `desc` are optional.

#### PageContext

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

The fully processed representation of a page. `pub_date` is formatted as an RFC 2822 string if `date` was present in the frontmatter.

---

## Feature Flags

| Feature    | Description                                                                 |
| ---------- | --------------------------------------------------------------------------- |
| `tera`     | Enables the built-in `TeraRenderer` (depends on `tera` crate)               |
| `pulldown` | Enables the built-in `PulldownMarkdown` (depends on `pulldown-cmark` crate) |
| `serve`    | Enables the dev server and file watcher (depends on `tiny_http`, `notify`)  |

All features are disabled by default. Enable them as needed.

---

## Security

- **Path confinement**: `safe_path` prevents directory traversal by canonicalizing and normalizing all paths.
- **Atomic output**: `Site::generate` writes to a temporary directory and renames it only on success, leaving the previous output intact on failure.
- **Strict configuration**: unknown YAML keys are rejected; content type patterns are validated via `glob::Pattern`.
- **No symlink following**: the library does not follow symbolic links; all operations use canonical paths.

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

A comprehensive mock filesystem (`MockFs`) and mock renderers are provided in `tests/common/mod.rs` for your own integration tests.

---

## Contributing

Please read [`CONTRIBUTING`](CONTRIBUTING) for guidelines on code style, commit messages, and the pull request process.

---

## License

This project is licensed under the [MIT License](LICENSE).
