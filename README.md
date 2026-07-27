# librawssg · [![GitHub tag](https://img.shields.io/github/v/tag/mroczect/librawssg?label=version)](https://github.com/mroczect/librawssg/tags) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![CI](https://github.com/mroczect/librawssg/actions/workflows/ci.yml/badge.svg)](https://github.com/mroczect/librawssg/actions/workflows/ci.yml)

**librawssg** is the engine‑agnostic, safety‑first kernel for building static site generators in Rust.  
It gives you all the primitives you need: filesystem abstraction, frontmatter parsing, Markdown rendering, template rendering, content processing pipelines, feed & sitemap generation, and a secure development server.

The library does **not** include a CLI – you write your own `main.rs` and compose the parts you need.  
Optional built‑in implementations for **Tera** and **pulldown‑cmark** are available behind feature flags.

---

## Table of Contents

- [What's New in v0.5.0](#whats-new-in-v050)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [API Reference](#api-reference)
  - [Configuration](#configuration)
    - [YAML Configuration File](#yaml-configuration-file)
    - [ConfigLoader trait](#configloader-trait)
    - [RawssgConfig::validate](#rawssgconfigvalidate)
    - [RawssgConfig default](#rawssgconfig-default)
  - [Error Handling](#error-handling)
  - [Filesystem Abstraction](#filesystem-abstraction)
    - [FileSystem trait](#filesystem-trait)
    - [RealFs](#realfs)
    - [Implementing a Custom FileSystem](#implementing-a-custom-filesystem)
  - [Markdown Rendering](#markdown-rendering)
    - [MarkdownRenderer trait](#markdownrenderer-trait)
    - [PulldownMarkdown](#pulldownmarkdown)
    - [Implementing a Custom Markdown Renderer](#implementing-a-custom-markdown-renderer)
  - [Template Rendering](#template-rendering)
    - [TemplateRenderer trait](#templaterenderer-trait)
    - [Context trait](#context-trait)
    - [TeraRenderer](#terarenderer)
    - [Implementing a Custom Template Engine](#implementing-a-custom-template-engine)
  - [Content Pipeline](#content-pipeline)
    - [ContentHandler trait](#contenthandler-trait)
    - [MarkdownPageHandler](#markdownpagehandler)
    - [StaticFileHandler](#staticfilehandler)
    - [build_page_context](#build_page_context)
    - [Adding Custom Handlers](#adding-custom-handlers)
  - [Site Builder](#site-builder)
    - [SiteBuilder](#sitebuilder)
    - [Site](#site)
    - [Atomic Generation & Cross-Device Fallback](#atomic-generation--cross-device-fallback)
  - [Feed & Sitemap](#feed--sitemap)
    - [generate_feed and generate_sitemap](#generate_feed-and-generate_sitemap)
    - [Context Builders](#context-builders)
  - [Utility Functions](#utility-functions)
    - [safe_path](#safe_path)
    - [slugify](#slugify)
    - [relative_prefix](#relative_prefix)
    - [match_pattern](#match_pattern)
  - [Type Reference](#type-reference)
    - [RawssgConfig](#rawssgconfig)
    - [GlobalConfig](#globalconfig)
    - [BuildConfig](#buildconfig)
    - [ContentTypeDef](#contenttypedef)
    - [GeneratorsConfig & GeneratorDef](#generatorsconfig--generatordef)
    - [NavItem](#navitem)
    - [PageFrontMatter](#pagefrontmatter)
    - [PageContext](#pagecontext)
  - [Dev Server & Watcher (serve feature)](#dev-server--watcher-serve-feature)
- [Feature Flags](#feature-flags)
- [Security](#security)
- [Full Customisation](#full-customisation)
  - [Step‑by‑Step: Building a Fully Custom SSG](#stepbystep-building-a-fully-custom-ssg)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

---

## What's New in v0.5.0

- **Robust path security** – Symlink‑safe output writing via canonicalised parent directories.
- **Correct glob matching** – `**` patterns now match exactly as expected (e.g., `blog/**/*.html` no longer matches `.md`).
- **Completely trait‑based** – Added `rename` to `FileSystem`; all I/O goes through the trait for full mockability.
- **Better defaults** – A default `page` content type (`**/*.md` → `base.html`) is included out‑of‑the‑box.
- **Optional context builders** – Feed and sitemap context builders are now only required when the respective generator is enabled.
- **Clearer error messages** – Missing closing `---` in frontmatter is reported explicitly; config loading failures are logged.
- **Improved testing** – Property‑based tests, dynamic server ports, and a complete mock filesystem.

---

## Installation

### Method 1: `cargo add` (Git dependency – recommended)

```bash
cargo add --git https://github.com/mroczect/librawssg.git --tag v0.5.0 librawssg
cargo add --git https://github.com/mroczect/librawssg.git --tag v0.5.0 librawssg --features tera,pulldown
```

### Method 2: Manual `Cargo.toml` entry

```toml
[dependencies]
librawssg = { git = "https://github.com/mroczect/librawssg.git", tag = "v0.5.0" }
librawssg = { git = "https://github.com/mroczect/librawssg.git", tag = "v0.5.0", features = ["tera", "pulldown"] }
```

### Method 3: Path dependency (local development)

```bash
git clone https://github.com/mroczect/librawssg.git
cd librawssg
# in your project's Cargo.toml:
librawssg = { path = "../librawssg", features = ["tera", "pulldown"] }
```

---

## Quick Start

```rust
use librawssg::SiteBuilder;
use librawssg::site::TeraRenderer;
use librawssg::markdown::PulldownMarkdown;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tera = TeraRenderer::new();
    tera.add_raw_template("base.html", "<html><body>{{ page_content }}</body></html>")?;
    let md = PulldownMarkdown;

    let mut config = librawssg::RawssgConfig::default();
    config.build.content_dir = "content".into();
    config.build.output_dir = "dist".into();

    let site = SiteBuilder::new()
        .config(config)
        .with_template_renderer(Box::new(tera))
        .with_markdown_renderer(Box::new(md))
        .build()?;

    site.generate()?;
    Ok(())
}
```

---

## Architecture

```
src/
  config/           ConfigLoader trait, YamlConfigLoader, DefaultConfig
  error.rs          RawssgError (miette + thiserror)
  frontmatter.rs    YAML frontmatter extraction and Markdown rendering
  fs/               FileSystem trait, RealFs
  markdown.rs       MarkdownRenderer trait, optional PulldownMarkdown
  serve/            Dev server and file watcher (feature "serve")
  site/
    builders/       Site and SiteBuilder structs
    context.rs      FeedContextBuilder, SitemapContextBuilder traits
    feed.rs         generate_feed
    mod.rs          Core traits: TemplateRenderer, Context, ContentHandler
    page.rs         build_page_context
    sitemap.rs      generate_sitemap
  types.rs          All configuration and page context types
  util.rs           safe_path, slugify, relative_prefix, match_pattern
```

---

## API Reference

### Configuration

#### YAML Configuration File

```yaml
site:
  site_name: "My Site"
  description: "A blog about Rust"
  base_url: "https://example.com"
  language: "en"
  author: "Alice"
build:
  content_dir: content
  output_dir: dist
  templates_dir: templates
  static_dir: static
content_types:
  - name: blog
    pattern: blog/**/*.md
    template: post.html
    list_template: blog_list.html
    list_enabled: true
  - name: page
    pattern: **/*.md
    template: page.html
generators:
  rss:
    enabled: true
    path: feed.xml
    template: rss.xml
  sitemap:
    enabled: true
    path: sitemap.xml
    template: sitemap.xml
```

#### ConfigLoader trait

```rust
pub trait ConfigLoader: Send + Sync {
    fn load(&self) -> Result<RawssgConfig, RawssgError>;
    fn load_or_default(&self) -> RawssgConfig;
}
```

- `YamlConfigLoader<P: AsRef<Path>>` – reads a YAML file.
- `DefaultConfig` – returns `RawssgConfig::default()`.

#### RawssgConfig::validate

- Checks that `site_name` is non‑empty.
- At least one `content_types` entry must exist.
- Each content type must have a valid glob pattern and a template name.
- If RSS or sitemap is enabled, their `path` and `template` must be set.

#### RawssgConfig default

Since v0.5.0, the default configuration includes a single content type:

```rust
ContentTypeDef {
    name: "page".into(),
    pattern: "**/*.md".into(),
    template: "base.html".into(),
    list_template: None,
    list_enabled: false,
}
```

You can remove it with `config.content_types.clear()` and define your own.

### Error Handling

`RawssgError` implements `std::error::Error`, `Display`, and `miette::Diagnostic`.

```rust
match err {
    RawssgError::Frontmatter { path, source } => { /* ... */ }
    RawssgError::PathTraversal(msg) => { /* ... */ }
    // ...
}
```

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
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>; // new in v0.5.0
}
```

#### RealFs

Default implementation delegating to `std::fs` and `walkdir`.

#### Implementing a Custom FileSystem

```rust
struct MyFs;
impl FileSystem for MyFs {
    // implement all methods; e.g., read from database or network
    fn read_to_string(&self, path: &Path) -> io::Result<String> { /* ... */ }
    // ... etc.
}
let site = SiteBuilder::new().with_fs(Box::new(MyFs)).build()?;
```

### Markdown Rendering

#### MarkdownRenderer trait

```rust
pub trait MarkdownRenderer: Send + Sync {
    fn render(&self, markdown: &str) -> String;
}
```

#### PulldownMarkdown

Available with `pulldown` feature. Enables tables, strikethrough, task lists.

#### Implementing a Custom Markdown Renderer

```rust
struct MyMd;
impl MarkdownRenderer for MyMd {
    fn render(&self, md: &str) -> String { my_parser(md) }
}
```

### Template Rendering

#### TemplateRenderer trait

```rust
pub trait TemplateRenderer: Send + Sync {
    fn render(&self, template_name: &str, context: &dyn Context) -> Result<String, RawssgError>;
}
```

#### Context trait

```rust
pub trait Context: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
```

#### TeraRenderer

- `new()` – creates an empty Tera instance.
- `add_raw_template(name, content)` – registers an inline template.

#### Implementing a Custom Template Engine

Implement `TemplateRenderer` and a `Context` wrapper.  
Example: MiniJinja.

```rust
struct MiniJinjaRenderer { env: mini_jinja::Environment<'static> }
impl TemplateRenderer for MiniJinjaRenderer {
    fn render(&self, name: &str, ctx: &dyn Context) -> Result<String, RawssgError> {
        let tmpl = self.env.get_template(name).map_err(|e| RawssgError::Template(e.to_string()))?;
        let data = ctx.as_any().downcast_ref::<serde_json::Value>().unwrap();
        tmpl.render(data).map_err(|e| RawssgError::Template(e.to_string()))
    }
}
impl Context for serde_json::Value { /* as_any downcast */ }
```

### Content Pipeline

#### ContentHandler trait

```rust
pub trait ContentHandler: Send + Sync {
    fn can_handle(&self, relative_path: &Path, original_path: &Path) -> bool;
    fn process(&self, fs: &dyn FileSystem, md_renderer: &dyn MarkdownRenderer,
               file_path: &Path, content_dir: &Path) -> Result<Option<PageContext>, RawssgError>;
}
```

Return `None` to skip a file.

#### MarkdownPageHandler

Handles `.md` files; extracts frontmatter, renders Markdown.

#### StaticFileHandler

Always returns `None` (catch‑all, non‑Markdown files become static assets).

#### build_page_context

```rust
pub fn build_page_context(fs: &dyn FileSystem, md_renderer: &dyn MarkdownRenderer,
    file_path: &Path, content_dir: &Path) -> Result<Option<PageContext>, RawssgError>;
```

Skips drafts. Returns a `PageContext` with URL, depth, date formatting.

#### Adding Custom Handlers

```rust
struct AsciiDocHandler;
impl ContentHandler for AsciiDocHandler {
    fn can_handle(&self, _rel: &Path, orig: &Path) -> bool {
        orig.extension().map_or(false, |e| e == "adoc")
    }
    fn process(&self, fs: &dyn FileSystem, _md: &dyn MarkdownRenderer, ...) -> Result<Option<PageContext>, RawssgError> {
        let content = fs.read_to_string(file_path)?;
        let html = asciidoc_render(&content);
        Ok(Some(PageContext { content_html: html, .. }))
    }
}
let builder = SiteBuilder::new().add_handler(Box::new(AsciiDocHandler));
```

### Site Builder

#### SiteBuilder

```rust
SiteBuilder::new()
    .config(config)
    .load_config("config.yml")?        // alternative to .config()
    .content_dir("my_content")
    .output_dir("public")
    .with_fs(Box::new(RealFs))
    .with_markdown_renderer(Box::new(PulldownMarkdown))
    .with_template_renderer(Box::new(TeraRenderer::new()))
    .with_feed_context_builder(Box::new(TeraFeedContextBuilder))
    .with_sitemap_context_builder(Box::new(TeraSitemapContextBuilder))
    .add_handler(Box::new(MyHandler))
    .build()?;
```

- `content_dir`, `output_dir` can be overridden by config values if left as default (`"content"`, `"dist"`).
- `feed_context_builder` and `sitemap_context_builder` are only required when the corresponding generator is enabled.

#### Site

```rust
let pages: &[PageContext] = site.pages();
site.generate()?;   // atomic write to output_dir
```

`generate()`:

1. Writes all pages (HTML) to `output_dir`.
2. Copies static assets from `static_dir`.
3. Copies non‑Markdown files from `content_dir`.
4. Optionally generates RSS and sitemap (if `tera` feature + enabled).
5. Uses atomic write: temp dir → rename (with cross‑device fallback).

#### Atomic Generation & Cross-Device Fallback

If `rename` fails with `CrossesDevices`, the library performs a recursive copy and then deletes the temporary directory.

### Feed & Sitemap

#### generate_feed and generate_sitemap

```rust
pub fn generate_feed(renderer: &dyn TemplateRenderer, config: &RawssgConfig,
    posts: &[&PageContext], base_url: &str, context_builder: &dyn FeedContextBuilder) -> Result<String, RawssgError>;
pub fn generate_sitemap(renderer: &dyn TemplateRenderer, config: &RawssgConfig,
    pages: &[PageContext], base_url: &str, context_builder: &dyn SitemapContextBuilder) -> Result<String, RawssgError>;
```

Callers are responsible for writing the returned string to the output file; `Site::generate` does this automatically.

#### Context Builders

```rust
pub trait FeedContextBuilder: Send + Sync {
    fn build_feed_context(&self, config: &RawssgConfig, posts: &[&PageContext], base_url: &str)
        -> Result<Box<dyn Context>, RawssgError>;
}
```

Default Tera implementations insert `site`, `posts`/`pages`, and `base_url`. Custom builders can add extra variables (e.g., `ctx.insert("custom", &"value")`).

### Utility Functions

#### safe_path

```rust
pub fn safe_path(fs: &dyn FileSystem, base: &Path, candidate: &Path) -> Result<PathBuf, RawssgError>;
```

- Canonicalises `base`.
- For existing files: canonicalises the candidate, checks it stays inside `base`.
- For non‑existent files (output): canonicalises the parent directory and verifies confinement.
- Returns an absolute, safe path.

#### slugify

Converts a string to lowercase, alphanumeric + hyphens. Example: `"Hello World!"` → `"hello-world"`.

#### relative_prefix

Returns `"./"` for depth 0, `"../"` repeated for deeper paths.

#### match_pattern

Glob matching with `*` (single segment) and `**` (multi‑segment). Correctly handles patterns like `blog/**/*.html`.

### Type Reference

#### RawssgConfig

```rust
pub struct RawssgConfig {
    pub site: GlobalConfig,
    pub build: BuildConfig,
    pub content_types: Vec<ContentTypeDef>,
    pub generators: GeneratorsConfig,
}
```

- `validate()` ensures invariants.
- `Default` now includes one content type.

#### GlobalConfig

```rust
pub struct GlobalConfig {
    pub site_name: String,            // default "rawssg"
    pub description: Option<String>,
    pub language: Option<String>,     // default Some("en")
    pub base_url: Option<String>,
    pub author: Option<String>,
    pub repo_url: Option<String>,
    pub license: Option<String>,
    pub navbar: Vec<NavItem>,
    pub sidebar: Vec<NavItem>,
}
```

#### BuildConfig

```rust
pub struct BuildConfig {
    pub content_dir: String,    // default "content"
    pub output_dir: String,     // default "dist"
    pub templates_dir: String,  // default "templates"
    pub static_dir: String,     // default "static"
}
```

#### ContentTypeDef

```rust
pub struct ContentTypeDef {
    pub name: String,
    pub pattern: String,          // glob
    pub template: String,
    pub list_template: Option<String>,
    pub list_enabled: bool,
}
```

#### GeneratorsConfig & GeneratorDef

```rust
pub struct GeneratorsConfig { pub rss: GeneratorDef, pub sitemap: GeneratorDef }
pub struct GeneratorDef {
    pub enabled: bool,      // default false
    pub path: String,
    pub template: String,
}
```

#### NavItem

```rust
pub struct NavItem { pub label: String, pub url: String }
```

#### PageFrontMatter

```rust
pub struct PageFrontMatter {
    pub title: String,
    pub desc: String,
    pub author: Option<String>,
    pub date: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub draft: bool,
    // ...
}
```

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

### Dev Server & Watcher (serve feature)

```rust
use librawssg::serve::start_dev_server;
start_dev_server(Path::new("dist"), 8080)?;
```

- Serves files with correct MIME types (including `text/plain` for `.txt`, `.md`, `.yaml`).
- 404 for missing, 500 for internal errors.

```rust
use librawssg::serve::watch_dirs;
let _watcher = watch_dirs(&[content_path, templates_path], || { rebuild(); })?;
```

Uses `notify` to trigger on `Modify`, `Create`, `Remove`.

---

## Feature Flags

| Feature    | Deps                 | Description                      |
| ---------- | -------------------- | -------------------------------- |
| `tera`     | `tera`               | `TeraRenderer`, context builders |
| `pulldown` | `pulldown-cmark`     | `PulldownMarkdown`               |
| `serve`    | `tiny_http`,`notify` | Dev server + file watcher        |

All disabled by default.

---

## Security

- **Path confinement**: `safe_path` prevents directory traversal for both existing and new files.
- **Atomic output**: Temp directory → rename; fallback copy‑and‑delete ensures atomicity across devices.
- **Configuration validation**: All YAML keys are known; glob patterns are validated.
- **Trait‑based I/O**: Every disk access goes through `FileSystem`, allowing sandboxing and auditing.

---

## Full Customisation

Every core component is a trait. You can:

- **Filesystem**: Implement `FileSystem` to read from database, in‑memory store, or network.
- **Markdown**: Any parser via `MarkdownRenderer`.
- **Templates**: Any engine via `TemplateRenderer` + `Context`.
- **Content handlers**: Add new file processors (e.g., AsciiDoc, reStructuredText).
- **Feed/Sitemap**: Custom context builders inject arbitrary variables.
- **Site generation**: Use `SiteBuilder` to build `Site`, then replace `generate()` with your own logic.

### Step‑by‑Step: Building a Fully Custom SSG

1. **Define your custom types** – implement the required traits.
2. **Load configuration** – use `YamlConfigLoader` or build `RawssgConfig` programmatically.
3. **Instantiate `SiteBuilder`** with your custom implementations.
4. **Call `.build()`** to obtain a `Site`.
5. **Generate** using `site.generate()`, or iterate over `site.pages()` for custom output.

---

## Testing

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

The test suite includes:

- `MockFs` – full mock filesystem (with `rename`).
- Mock renderers.
- Property‑based tests (`proptest`).
- Integration tests: full generation, drafts, blog lists, RSS/sitemap errors.
- Dynamic port allocation for server tests.

---

## Contributing

Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines.  
All contributions are welcome – issues, PRs, documentation improvements.

---

## License

MIT. See [`LICENSE`](LICENSE).
