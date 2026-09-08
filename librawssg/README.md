# librawssg

A modular static site generator library for Rust.

This facade crate re‑exports the essential building blocks from the `librawssg` ecosystem, providing a single convenient entry point for building static site generators. It aggregates configuration management, filesystem abstraction, content processing, template rendering (with Tera built‑in), and build pipeline orchestration.

---

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Modules](#modules)
4. [Core Types and Traits](#core-types-and-traits)
   - [Configuration](#configuration)
   - [Filesystem](#filesystem)
   - [Content Handling](#content-handling)
   - [Template Rendering](#template-rendering)
   - [Compiler Pipeline](#compiler-pipeline)
   - [Error Handling](#error-handling)
5. [Usage Example](#usage-example)
6. [Full API Reference](#full-api-reference)
   - [Configuration Types](#configuration-types)
   - [Filesystem Types](#filesystem-types)
   - [Handler Types](#handler-types)
   - [Template Types](#template-types)
   - [Compiler Types](#compiler-types)
   - [Error Types](#error-types)
7. [Feature Flags](#feature-flags)
8. [License](#license)

---

## Overview

`librawssg` is the top‑level crate that brings together six specialized crates:

- **`librawssg_config`** – Configuration data structures and validation.
- **`librawssg_fs`** – Trait‑based filesystem abstraction with path traversal protection.
- **`librawssg_handler`** – Core document and metadata types, plus the `Processor` trait.
- **`librawssg_templates`** – Rendering traits (`Renderer`, `RenderContext`) and a Tera implementation.
- **`librawssg_compiler`** – Build pipeline orchestration.
- **`librawssg_error`** – Unified error enum and `Result` alias.

By depending on `librawssg`, you get all these components without needing to specify each one individually. The facade also re‑exports the most commonly used types at the crate root for ergonomic access.

---

## Installation

Add `librawssg` to your `Cargo.toml`:

```toml
[dependencies]
librawssg = "1.0.0"
```

If you are working in the same workspace as the `librawssg` source, you can use a path dependency:

```toml
[dependencies]
librawssg = { path = "../librawssg" }
```

The crate is compatible with Rust edition 2024 and later.

---

## Modules

The crate organises its re‑exports into submodules for clarity:

- **`librawssg::config`** – Configuration types (`Config`, `SiteConfig`, `BuildConfig`, `ContentRule`, `NavItem`).
- **`librawssg::fs`** – Filesystem trait and `RealFs`.
- **`librawssg::handler`** – Document, metadata, and processor contracts.
- **`librawssg::templates`** – Rendering traits and `TeraRenderer`.
- **`librawssg::compiler`** – Pipeline builder, pipeline, context builders, generators.
- **`librawssg::error`** – Error type and `Result` alias.

Additionally, the most important types are also re‑exported directly at the crate root for convenience.

---

## Core Types and Traits

### Configuration

The configuration system revolves around the `Config` struct, which contains site settings, build paths, and content processing rules.

- **`Config`** – Top‑level configuration.
  - **Fields**: `site: SiteConfig`, `build: BuildConfig`, `content_rules: Vec<ContentRule>`, `extra: HashMap<String, serde_json::Value>`.
  - **Constructors**:
    - `Config::new() -> Config`
    - `Config::default() -> Config`
  - **Builder method**: `with_site_name(name: impl Into<String>) -> Self`
  - **Rule management**:
    - `add_content_rule(&mut self, rule: ContentRule)`
    - `find_rule_by_name(&self, name: &str) -> Option<&ContentRule>`
    - `remove_rule_by_name(&mut self, name: &str) -> Option<ContentRule>`
    - `has_duplicate_rule_names(&self) -> bool`
  - **Validation**: `validate(&self) -> Result<()>`
  - **Serialization**:
    - `from_yaml_str(yaml: &str) -> Result<Self>`
    - `to_yaml_string(&self) -> Result<String>`
    - `from_json_str(json: &str) -> Result<Self>`
    - `to_json_string(&self) -> Result<String>`

- **`SiteConfig`** – Global site metadata.
  - **Fields**: `navbar`, `sidebar`, `site_name`, `description`, `language`, `base_url`, `author`, `repo_url`, `license`, `extra`.
  - **Constructors**: `SiteConfig::new(site_name: impl Into<String>) -> Self`, `SiteConfig::default()`.
  - **Defaults**: `site_name = "librawssg"`, `language = Some("en")`.

- **`BuildConfig`** – Filesystem path settings.
  - **Fields**: `content_dir`, `output_dir`, `templates_dir`, `static_dir`.
  - **Constructors**: `BuildConfig::new()`, `BuildConfig::default()`.
  - **Defaults**: `"content"`, `"dist"`, `"templates"`, `"static"`.

- **`ContentRule`** – Defines how a group of files should be processed.
  - **Fields**: `name`, `pattern`, `template`, `list_template: Option<String>`, `list_enabled: bool`, `extra: HashMap<String, serde_json::Value>`.
  - **Constructor**: `ContentRule::new(name, pattern, template) -> Self`.
  - **Default**: all strings empty, `list_enabled = false`.

- **`NavItem`** – Navigation menu entry.
  - **Fields**: `label: String`, `url: String`, `children: Vec<NavItem>`.
  - **Constructor**: `NavItem::new(label, url) -> Self`.

### Filesystem

- **`FileSystem` trait** – Abstract filesystem operations. All methods return `io::Result` or `bool`.
  - Required methods:
    - `read_to_string`, `read_bytes`, `write`, `create_dir_all`, `remove_dir_all`, `remove_file`, `create_dir`, `exists`, `is_dir`, `is_file`, `read_dir`, `copy_file`, `copy_dir_all`, `walk_dir`, `canonicalize`, `rename`, `atomic_write`, `touch`, `metadata`, `symlink_metadata`, `permissions`, `set_permissions`, `read_link`, `hard_link`.
  - Provided methods (with default implementations):
    - `is_symlink`
    - `canonicalize_or_join`
    - `safe_join` – **Important for security**: ensures the resulting path stays within a base directory.
    - `copy`
    - `rename_or_copy`

- **`RealFs`** – Zero‑sized struct implementing `FileSystem` using `std::fs` and `walkdir`.

### Content Handling

- **`Document`** – Represents a processed content item.
  - **Fields**: `metadata: Metadata`, `body: String`, `url: String`, `output_path: PathBuf`, `source_path: PathBuf`, `depth: usize`, `content_type: String`, `is_list: bool`, `list_items: Option<Vec<Document>>`, `taxonomies: HashMap<String, Vec<String>>`.
  - **Constructor**: `Document::new(metadata, body, url, output_path, source_path, depth, content_type, is_list) -> Result<Self>`.
  - **Methods**:
    - `relative_url(&self) -> &str`
    - `add_taxonomy(&mut self, name: impl Into<String>, items: Vec<String>)`
    - `depth(&self) -> usize`
    - `with_list_items(self, items: Vec<Self>) -> Self`

- **`Metadata`** – Front matter data.
  - **Fields**: `title`, `description`, `author: Option<String>`, `repo_url`, `license`, `date: Option<NaiveDate>`, `updated: Option<NaiveDate>`, `tags: Vec<String>`, `draft: bool`, `extra: HashMap<String, serde_json::Value>`.
  - **Constructor**: `Metadata::new(title, description) -> Result<Self>`.
  - **Methods**:
    - `is_draft(&self) -> bool`
    - `insert_extra(&mut self, key, value)`
    - `get_extra(&self, key: &str) -> Option<&serde_json::Value>`

- **`Processor` trait** – Interface for transforming source files into `Document`s.
  - Required methods:
    - `name(&self) -> &str`
    - `can_process(&self, relative_path: &Path, original_path: &Path) -> bool`
    - `process(&self, fs: &dyn FileSystem, relative_path: &Path, content_dir: &Path) -> Result<Option<Document>>`
  - Provided method: `priority(&self) -> i32` (default 0).

### Template Rendering

- **`RenderContext` trait** – Type‑erased context for renderers.
  - Required methods: `as_any(&self) -> &dyn Any`, `as_mut_any(&mut self) -> &mut dyn Any`.

- **`Renderer` trait** – Interface for template rendering.
  - Required method: `render(&self, template_name: &str, context: &dyn RenderContext) -> Result<String>`.

- **`TeraRenderer`** – Concrete renderer using the Tera template engine.
  - Available when the `tera` feature is enabled (enabled by default).
  - **Constructors**: `TeraRenderer::new()`, `Default`.
  - **Methods**:
    - `add_raw_template(&mut self, name: &str, content: &str) -> Result<()>`
    - `add_template_file(&mut self, path: &Path) -> Result<()>`
    - `add_template_files_from_dir(&mut self, dir: &Path) -> Result<()>`
    - `load_templates_dir(&mut self, dir: &Path) -> Result<()>`
    - `enable_autoescape(&mut self)`
    - `render_str(&self, template_str: &str, context: &dyn RenderContext) -> Result<String>`
    - `as_tera(&self) -> &tera::Tera`
    - `as_tera_mut(&mut self) -> &mut tera::Tera`

### Compiler Pipeline

- **`PipelineBuilder`** – Builds a `Pipeline`.
  - **Constructor**: `PipelineBuilder::new()`.
  - **Builder methods**: `config`, `load_config`, `content_dir`, `output_dir`, `with_fs`, `with_renderer`, `add_processor`, `with_context_builder`, `add_generator`.
  - **Build**: `build(self) -> Result<Pipeline>`.

- **`Pipeline`** – Executes the site generation.
  - **Method**: `run(&self) -> Result<()>`.
  - **Accessor**: `config(&self) -> &Config`.

- **`ContextBuilder` trait** – Creates a `RenderContext` from `Config` and `Document`.
  - Required method: `build_context(&self, config: &Config, doc: &Document) -> Result<Box<dyn RenderContext>>`.

- **`TeraContextBuilder`** – Default implementation that produces a `tera::Context` with common page and site variables.

- **`Generator` trait** – Custom post‑processing step.
  - Required method: `generate(&self, pipeline: &Pipeline, output_base: &Path) -> Result<()>`.

- **`match_pattern` function** (in `compiler::pattern`) – Glob matching for content rule patterns.

### Error Handling

- **`Error` enum** – Variants: `Io`, `Config`, `Metadata`, `Render`, `Processor`, `Generator`, `PathTraversal`, `MissingConfig`, `Generation`, `NotFound`, `Serialization`, `Validation`, `Duplicate`, `InvalidState`, `Internal`.
- **`Result<T>`** – Alias for `core::result::Result<T, Error>`.

---

## Usage Example

Here’s a minimal but complete example that builds a site from raw HTML fragments using a custom processor, Tera templates, and the default context builder:

```rust
use librawssg::{
    Config, ContentRule, Document, FileSystem, Metadata, PipelineBuilder, Processor,
    RealFs, RenderContext, Renderer, TeraContextBuilder, TeraRenderer,
};
use std::path::{Path, PathBuf};

// 1. Define a simple processor for .raw files
struct RawProcessor;
impl Processor for RawProcessor {
    fn name(&self) -> &'static str { "raw" }
    fn can_process(&self, rel: &Path, _orig: &Path) -> bool {
        rel.extension().and_then(|e| e.to_str()) == Some("raw")
    }
    fn process(
        &self,
        fs: &dyn FileSystem,
        rel: &Path,
        content_dir: &Path,
    ) -> librawssg::Result<Option<Document>> {
        let full_path = content_dir.join(rel);
        let body = fs.read_to_string(&full_path)?;
        let title = rel.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let meta = Metadata::new(title, String::new())?;
        let url = rel.with_extension("html").to_string_lossy().to_string();
        let output = PathBuf::from(&url);
        let doc = Document::new(meta, body, url, output, rel.to_path_buf(), 0, "page".to_string(), false)?;
        Ok(Some(doc))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 2. Configure the site
    let mut config = Config::new().with_site_name("My Site");
    config.add_content_rule(ContentRule::new("page", "**/*.raw", "base.tera"));
    config.build.content_dir = "content".to_string();
    config.build.output_dir = "dist".to_string();
    config.build.static_dir = "static".to_string();

    // 3. Set up the renderer and load templates
    let mut renderer = TeraRenderer::new();
    renderer.load_templates_dir(Path::new("templates"))?;

    // 4. Build the pipeline
    let pipeline = PipelineBuilder::new()
        .config(config)
        .content_dir("content")
        .output_dir("dist")
        .with_fs(Box::new(RealFs))
        .with_renderer(Box::new(renderer))
        .with_context_builder(Box::new(TeraContextBuilder))
        .add_processor(Box::new(RawProcessor))
        .build()?;

    // 5. Run the generation
    pipeline.run()?;
    println!("Site generated successfully!");
    Ok(())
}
```

For more detailed examples, see the `librawssg_demo` crate in the repository.

---

## Full API Reference

This section provides a concise reference for every public item re‑exported by `librawssg`. For deeper details, consult the respective sub‑crate documentation (e.g., `librawssg_config`, `librawssg_compiler`).

### Configuration Types

All configuration types are in `librawssg::config` (and re‑exported at root).

- **`Config`**
  - `new() -> Self`
  - `default() -> Self`
  - `with_site_name(self, name: impl Into<String>) -> Self`
  - `add_content_rule(&mut self, rule: ContentRule)`
  - `find_rule_by_name(&self, name: &str) -> Option<&ContentRule>`
  - `remove_rule_by_name(&mut self, name: &str) -> Option<ContentRule>`
  - `has_duplicate_rule_names(&self) -> bool`
  - `validate(&self) -> Result<()>`
  - `from_yaml_str(yaml: &str) -> Result<Self>`
  - `to_yaml_string(&self) -> Result<String>`
  - `from_json_str(json: &str) -> Result<Self>`
  - `to_json_string(&self) -> Result<String>`

- **`SiteConfig`**
  - `new(site_name: impl Into<String>) -> Self`
  - `default() -> Self`
  - Fields: `navbar: Vec<NavItem>`, `sidebar: Vec<NavItem>`, `site_name: String`, `description: Option<String>`, `language: Option<String>`, `base_url: Option<String>`, `author: Option<String>`, `repo_url: Option<String>`, `license: Option<String>`, `extra: HashMap<String, serde_json::Value>`

- **`BuildConfig`**
  - `new() -> Self`
  - `default() -> Self`
  - Fields: `content_dir: String`, `output_dir: String`, `templates_dir: String`, `static_dir: String`

- **`ContentRule`**
  - `new(name: impl Into<String>, pattern: impl Into<String>, template: impl Into<String>) -> Self`
  - `default() -> Self`
  - Fields: `name: String`, `pattern: String`, `template: String`, `list_template: Option<String>`, `list_enabled: bool`, `extra: HashMap<String, serde_json::Value>`

- **`NavItem`**
  - `new(label: impl Into<String>, url: impl Into<String>) -> Self`
  - `default() -> Self`
  - Fields: `label: String`, `url: String`, `children: Vec<NavItem>`

### Filesystem Types

- **`FileSystem` trait** (in `librawssg::fs`, re‑exported at root)
  - Required methods (see above).
  - Provided methods: `is_symlink`, `canonicalize_or_join`, `safe_join`, `copy`, `rename_or_copy`.

- **`RealFs`** (in `librawssg::fs`, re‑exported at root)
  - Implements `FileSystem` using `std::fs`.
  - `RealFs` (unit struct), `RealFs::default()`.

### Handler Types

- **`Document`** (in `librawssg::handler`, re‑exported at root)
  - `new(metadata, body, url, output_path, source_path, depth, content_type, is_list) -> Result<Self>`
  - `relative_url(&self) -> &str`
  - `add_taxonomy(&mut self, name, items)`
  - `depth(&self) -> usize`
  - `with_list_items(self, items: Vec<Self>) -> Self`
  - Fields: as listed earlier.

- **`Metadata`**
  - `new(title, description) -> Result<Self>`
  - `is_draft(&self) -> bool`
  - `insert_extra(&mut self, key, value)`
  - `get_extra(&self, key: &str) -> Option<&serde_json::Value>`
  - Fields: as listed earlier.

- **`Processor` trait**
  - Required: `name`, `can_process`, `process`
  - Provided: `priority`

### Template Types

- **`RenderContext` trait**
  - `as_any(&self) -> &dyn Any`
  - `as_mut_any(&mut self) -> &mut dyn Any`

- **`Renderer` trait**
  - `render(&self, template_name: &str, context: &dyn RenderContext) -> Result<String>`

- **`TeraRenderer`** (feature `tera`, enabled by default)
  - `new() -> Self`
  - `add_raw_template(&mut self, name: &str, content: &str) -> Result<()>`
  - `add_template_file(&mut self, path: &Path) -> Result<()>`
  - `add_template_files_from_dir(&mut self, dir: &Path) -> Result<()>`
  - `load_templates_dir(&mut self, dir: &Path) -> Result<()>`
  - `enable_autoescape(&mut self)`
  - `render_str(&self, template_str: &str, context: &dyn RenderContext) -> Result<String>`
  - `as_tera(&self) -> &tera::Tera`
  - `as_tera_mut(&mut self) -> &mut tera::Tera`
  - Implements `Renderer` and `Default`.

### Compiler Types

- **`PipelineBuilder`**
  - `new() -> Self`
  - `config(self, config: Config) -> Self`
  - `load_config<P: AsRef<Path> + Send + Sync>(self, path: P) -> Result<Self>`
  - `content_dir(self, dir: impl Into<PathBuf>) -> Self`
  - `output_dir(self, dir: impl Into<PathBuf>) -> Self`
  - `with_fs(self, fs: Box<dyn FileSystem>) -> Self`
  - `with_renderer(self, renderer: Box<dyn Renderer>) -> Self`
  - `add_processor(self, processor: Box<dyn Processor>) -> Self`
  - `with_context_builder(self, builder: Box<dyn ContextBuilder>) -> Self`
  - `add_generator(self, generator: Box<dyn Generator>) -> Self`
  - `build(self) -> Result<Pipeline>`

- **`Pipeline`**
  - `run(&self) -> Result<()>`
  - `config(&self) -> &Config`

- **`ContextBuilder` trait**
  - `build_context(&self, config: &Config, doc: &Document) -> Result<Box<dyn RenderContext>>`

- **`TeraContextBuilder`** (unit struct)
  - Implements `ContextBuilder`.
  - `TeraContextBuilder` (no fields), `Default`.

- **`Generator` trait**
  - `generate(&self, pipeline: &Pipeline, output_base: &Path) -> Result<()>`

- **`match_pattern` function** (accessible via `librawssg::compiler::pattern::match_pattern`)
  - Signature: `pub fn match_pattern(pattern: &str, path: &Path) -> bool`
  - Supports glob patterns with `*` and `**`.

### Error Types

- **`Error` enum** (in `librawssg::error`, re‑exported at root)
  - Variants: as listed above.
  - Implements `Display`, `Debug`, `std::error::Error`.
  - `From<std::io::Error>` is implemented.

- **`Result<T>`** type alias
  - `pub type Result<T> = core::result::Result<T, Error>`

---

## Feature Flags

The `tera` feature is enabled by default and provides the `TeraRenderer` implementation. To disable it (e.g., if you use a different template engine), set `default-features = false` in your `Cargo.toml`:

```toml
[dependencies]
librawssg = { version = "1.0.0", default-features = false }
```

Without this feature, the crate still exports the core traits (`Renderer`, `RenderContext`) and all other functionality, but `TeraRenderer` and `TeraContextBuilder` are unavailable.

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

---

_This documentation is generated from the source code of the `librawssg` facade crate and its sub‑crates._
