# librawssg_compiler

**Version**: 1.0.0 (implied)  
**Crate name**: `librawssg_compiler`  
**Description**: Provides the core compilation pipeline for the `librawssg` static site generator. This crate orchestrates the entire build process: reading content files, processing them via pluggable processors, rendering templates, copying static assets, running custom generators, and outputting the final site atomically.

---

## Table of Contents

1. [Overview](#overview)
2. [Modules and Re‑exports](#modules-and-re-exports)
3. [`PipelineBuilder`](#struct-pipelinebuilder)
   - [Constructor `new()`](#pipelinebuilder-new)
   - [Builder Methods](#pipelinebuilder-builder-methods)
     - [`config()`](#pipelinebuilder-config)
     - [`load_config()`](#pipelinebuilder-load_config)
     - [`content_dir()`](#pipelinebuilder-content_dir)
     - [`output_dir()`](#pipelinebuilder-output_dir)
     - [`with_fs()`](#pipelinebuilder-with_fs)
     - [`with_renderer()`](#pipelinebuilder-with_renderer)
     - [`add_processor()`](#pipelinebuilder-add_processor)
     - [`with_context_builder()`](#pipelinebuilder-with_context_builder)
     - [`add_generator()`](#pipelinebuilder-add_generator)
   - [`build()` Method](#pipelinebuilder-build)
   - [`Default` Implementation](#pipelinebuilder-default)
   - [Example Usage](#pipelinebuilder-example)
4. [`ContextBuilder` Trait](#trait-contextbuilder)
   - [Required Method `build_context()`](#contextbuilder-build_context)
   - [`TeraContextBuilder`](#struct-teracontextbuilder)
     - [Implementation Details](#teracontextbuilder-implementation)
5. [`Generator` Trait](#trait-generator)
   - [Required Method `generate()`](#generator-generate)
6. [Pattern Matching Function](#function-match_pattern)
   - [Signature](#match_pattern-signature)
   - [Supported Glob Syntax](#match_pattern-syntax)
   - [Algorithm Overview](#match_pattern-algorithm)
   - [Examples](#match_pattern-examples)
7. [`Pipeline` Struct](#struct-pipeline)
   - [Accessor `config()`](#pipeline-config)
   - [Method `run()`](#pipeline-run)
   - [Internal Workflow](#pipeline-internal-workflow)
     - [Document Processing](#pipeline-document-processing)
     - [Content Type Determination](#pipeline-content-type)
     - [Rendering](#pipeline-rendering)
     - [List Generation](#pipeline-list-generation)
     - [Static Assets](#pipeline-static-assets)
     - [Generators](#pipeline-generators)
     - [Atomic Output Replacement](#pipeline-atomic-output)
8. [Error Handling](#error-handling)
9. [Complete Example from Tests](#complete-example-from-tests)
   - [Setting Up a Pipeline](#example-setup)
   - [Running the Pipeline](#example-run)
   - [Verifying Output](#example-verify)
10. [Testing Suite Overview](#testing-suite-overview)
11. [Conclusion](#conclusion)

---

## Overview

`librawssg_compiler` is the orchestration layer that ties together all other components of the static site generator:

- **`Config`** from `librawssg_config` defines content rules and paths.
- **`Processor`** from `librawssg_handler` transforms source files into `Document` objects.
- **`Renderer`** and **`RenderContext`** from `librawssg_templates` handle template rendering.
- **`FileSystem`** from `librawssg_fs` abstracts all I/O operations.
- **`Generator`** (defined here) allows custom post‑processing steps.
- **`ContextBuilder`** (defined here) constructs the render context from a `Document` and `Config`.

The main entry point is `PipelineBuilder`, which constructs a `Pipeline` after validating the configuration and ensuring mandatory components are present. Running the pipeline performs the full site generation in an atomic fashion, producing output in the configured output directory.

---

## Modules and Re‑exports

The crate root (`lib.rs`) declares the following public modules:

- `builder` – Contains `PipelineBuilder`.
- `context` – Contains `ContextBuilder` trait and `TeraContextBuilder`.
- `generator` – Contains `Generator` trait.
- `pattern` – Contains `match_pattern` function.
- `pipeline` – Contains `Pipeline` struct.

Re‑exported types at the crate root:

```rust
pub use builder::PipelineBuilder;
pub use context::ContextBuilder;
pub use context::TeraContextBuilder;
pub use generator::Generator;
pub use pipeline::Pipeline;
```

The `match_pattern` function is also re‑exported? Actually it is in `pattern` module and not re‑exported at root, so users must use `librawssg_compiler::pattern::match_pattern`. However, in `pipeline.rs` it is imported via `crate::pattern::match_pattern`, but for external users they need to access it via module path.

---

## Struct `PipelineBuilder`

`PipelineBuilder` is a builder‑style struct that collects all components needed to run the compilation pipeline and then builds a `Pipeline`.

```rust
pub struct PipelineBuilder {
    config: Config,
    content_dir: PathBuf,
    output_dir: PathBuf,
    fs: Box<dyn FileSystem>,
    renderer: Option<Box<dyn Renderer>>,
    processors: Vec<Box<dyn Processor>>,
    context_builder: Option<Box<dyn ContextBuilder>>,
    generators: Vec<Box<dyn Generator>>,
}
```

**Note**: Fields are private; use the builder methods to configure.

### `PipelineBuilder::new`

```rust
#[must_use]
pub fn new() -> Self
```

**Purpose**: Creates a new builder with default values:

- `config`: `Config::default()`
- `content_dir`: `"content"`
- `output_dir`: `"dist"`
- `fs`: `Box::new(RealFs)`
- `renderer`: `None`
- `processors`: empty vector
- `context_builder`: `None`
- `generators`: empty vector

**Returns**: A fresh `PipelineBuilder`.

**Example**:

```rust
let builder = PipelineBuilder::new();
```

---

### PipelineBuilder Builder Methods

All builder methods consume `self` and return `Self`, allowing method chaining.

#### `config`

```rust
#[must_use]
pub fn config(mut self, config: Config) -> Self
```

**Purpose**: Sets the configuration object.

**Parameters**:

- `config`: A `Config` instance from `librawssg_config`.

**Returns**: The builder with the config set.

#### `load_config`

```rust
pub fn load_config<P: AsRef<Path> + Send + Sync>(mut self, path: P) -> Result<Self>
```

**Purpose**: Reads a YAML config file from disk and parses it into a `Config`. Errors are converted to `Error::Config`.

**Parameters**:

- `path`: Path to the YAML file.

**Returns**: `Ok(Self)` if parsing succeeds, otherwise `Err(Error::Config)`.

**Note**: The file is read using standard `std::fs::read_to_string`; the error is wrapped in `Error::Config`.

#### `content_dir`

```rust
#[must_use]
pub fn content_dir(mut self, dir: impl Into<PathBuf>) -> Self
```

**Purpose**: Overrides the content directory.

**Parameters**:

- `dir`: Any type convertible to `PathBuf`.

**Default**: `"content"` (but may be overridden by config if not explicitly set; see `build()`).

#### `output_dir`

```rust
#[must_use]
pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self
```

**Purpose**: Overrides the output directory.

**Default**: `"dist"`.

#### `with_fs`

```rust
#[must_use]
pub fn with_fs(mut self, fs: Box<dyn FileSystem>) -> Self
```

**Purpose**: Sets a custom filesystem implementation. Useful for testing or non‑standard backends.

**Default**: `RealFs`.

#### `with_renderer`

```rust
#[must_use]
pub fn with_renderer(mut self, renderer: Box<dyn Renderer>) -> Self
```

**Purpose**: Sets the template renderer. **Mandatory**; `build()` will fail if not set.

#### `add_processor`

```rust
#[must_use]
pub fn add_processor(mut self, processor: Box<dyn Processor>) -> Self
```

**Purpose**: Adds a content processor to the pipeline. Multiple processors can be added; they are tried in order for each source file.

#### `with_context_builder`

```rust
#[must_use]
pub fn with_context_builder(mut self, builder: Box<dyn ContextBuilder>) -> Self
```

**Purpose**: Sets the context builder. **Mandatory**; `build()` will fail if not set.

#### `add_generator`

```rust
#[must_use]
pub fn add_generator(mut self, generator: Box<dyn Generator>) -> Self
```

**Purpose**: Adds a post‑processing generator. Generators run after all documents are rendered and static assets copied.

---

### PipelineBuilder::build

```rust
pub fn build(mut self) -> Result<Pipeline>
```

**Purpose**: Validates the configuration, ensures required components are present, and constructs a `Pipeline`.

**Behavior**:

1. Calls `self.config.validate()?` (see `librawssg_config::Config::validate`).
2. If `content_dir` is still the default `"content"` (i.e., not changed by `content_dir()`), it is replaced with `self.config.build.content_dir`.
3. Similarly, if `output_dir` is still `"dist"`, it is replaced with `self.config.build.output_dir`.
4. Takes the renderer from `self.renderer` (using `take()`). If `None`, returns `Error::Config("template renderer not set")`.
5. Takes the context builder from `self.context_builder`. If `None`, returns `Error::Config("context builder not set")`.
6. Moves all remaining fields into a new `Pipeline` and returns `Ok`.

**Returns**:

- `Ok(Pipeline)` on success.
- `Err(Error::Validation)` if config invalid.
- `Err(Error::Config)` if renderer or context builder missing.

---

### PipelineBuilder Default

```rust
impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

Allows creating with `PipelineBuilder::default()`.

---

### PipelineBuilder Example

```rust
use librawssg_compiler::{PipelineBuilder, TeraContextBuilder};
use librawssg_config::Config;
use librawssg_fs::RealFs;
use librawssg_handler::Processor;
use librawssg_templates::{Renderer, TeraRenderer};

// Assuming custom processor and renderer exist
let config = Config::new().with_site_name("My Site");
let processor = Box::new(MyProcessor);
let renderer = Box::new(TeraRenderer::new()); // TeraRenderer must be configured with templates beforehand
let context_builder = Box::new(TeraContextBuilder);

let pipeline = PipelineBuilder::new()
    .config(config)
    .content_dir("src")
    .output_dir("public")
    .with_fs(Box::new(RealFs))
    .with_renderer(renderer)
    .with_context_builder(context_builder)
    .add_processor(processor)
    .build()?;
```

---

## Trait `ContextBuilder`

```rust
pub trait ContextBuilder: Send + Sync {
    fn build_context(&self, config: &Config, doc: &Document) -> Result<Box<dyn RenderContext>>;
}
```

**Purpose**: Abstract factory that creates a `RenderContext` from the global `Config` and a specific `Document`. The renderer then uses this context to render the document's template.

**Requirements**: Implementors must be `Send + Sync`.

### `build_context`

**Parameters**:

- `config`: Reference to the site configuration.
- `doc`: Reference to the document being rendered.

**Returns**:

- `Ok(Box<dyn RenderContext>)` – a boxed trait object holding the render context.
- `Err(librawssg_error::Error)` if context creation fails.

---

## Struct `TeraContextBuilder`

```rust
#[derive(Debug, Default, Clone, Copy)]
pub struct TeraContextBuilder;
```

A concrete implementation of `ContextBuilder` that produces a `tera::Context` populated with common page and site data.

### Implementation Details

`TeraContextBuilder` creates a new `tera::Context` and inserts the following keys:

| Key                | Value Source                | Description                                   |
| ------------------ | --------------------------- | --------------------------------------------- |
| `site`             | `&config.site`              | The full `SiteConfig` object.                 |
| `page_title`       | `&doc.metadata.title`       | Document title.                               |
| `page_description` | `&doc.metadata.description` | Document description.                         |
| `page_author`      | `&doc.metadata.author`      | Optional author.                              |
| `page_date`        | `&doc.metadata.date`        | Optional publication date.                    |
| `page_tags`        | `&doc.metadata.tags`        | Vector of tags.                               |
| `page_content`     | `&doc.body`                 | The rendered body content of the document.    |
| `page_url`         | `&doc.url`                  | Relative URL of the document.                 |
| `page_depth`       | `&doc.depth`                | Depth in the site hierarchy.                  |
| `page_type`        | `&doc.content_type`         | Content type identifier (e.g., `"blog"`).     |
| `page_is_list`     | `&doc.is_list`              | Boolean indicating list page.                 |
| `page_list_items`  | `&doc.list_items`           | Optional vector of child documents for lists. |

**Note**: The `page_list_items` field is inserted as `&doc.list_items` which is `Option<Vec<Document>>`. In Tera templates, this will be `None` or an array.

**Example**:

```rust
use librawssg_compiler::TeraContextBuilder;
let builder = TeraContextBuilder;
let ctx = builder.build_context(&config, &doc)?;
// Pass `ctx` to renderer.render(...)
```

---

## Trait `Generator`

```rust
pub trait Generator: Send + Sync {
    fn generate(&self, pipeline: &Pipeline, output_base: &Path) -> Result<()>;
}
```

**Purpose**: Allows custom post‑processing steps after the main site generation. Generators can write additional files to the output directory (e.g., RSS feed, sitemap, search index).

**Requirements**: Implementors must be `Send + Sync`.

### `generate`

**Parameters**:

- `pipeline`: Reference to the running `Pipeline`, which provides access to its configuration, filesystem, etc. (though the fields are crate‑private, the `config()` method is available).
- `output_base`: Path to the temporary output directory where generated files should be written. The pipeline's `run()` method later moves this directory atomically to the final output location.

**Returns**:

- `Ok(())` on success.
- `Err(librawssg_error::Error)` on failure.

**Example** (from tests):

```rust
struct DummyGenerator;

impl Generator for DummyGenerator {
    fn generate(&self, _pipeline: &Pipeline, output_base: &Path) -> Result<()> {
        let path = output_base.join("generated.txt");
        std::fs::write(&path, b"generated")?;
        Ok(())
    }
}
```

---

## Function `match_pattern`

```rust
#[must_use]
pub fn match_pattern(pattern: &str, path: &Path) -> bool
```

**Location**: `librawssg_compiler::pattern`

**Purpose**: Checks whether a file path matches a glob pattern with support for `*` (within a segment) and `**` (across segments). Used by the pipeline to assign content types based on `ContentRule` patterns.

**Parameters**:

- `pattern`: A glob‑like pattern string, e.g., `"blog/**/*.html"`.
- `path`: A `Path` to test (usually a relative path from the content directory).

**Returns**: `true` if the path matches the pattern; `false` otherwise.

### Supported Glob Syntax

- `*` – Matches any sequence of characters within a single path segment (i.e., does not cross `/`).
- `**` – Matches any number of path segments, including zero.

**Limitations**:

- Only `*` and `**` are supported; no character classes (`[abc]`) or alternation.
- Patterns are split on `/`; backslashes are not treated as separators (path normalization may be needed on Windows).
- The implementation uses a custom recursive algorithm; for complex patterns, behavior may differ from standard glob libraries.

### Algorithm Overview

The function first converts the path to a string (lossy) and splits both pattern and path by `/`. It then calls an internal recursive `match_pattern_slice`. The logic:

- If both pattern and path segments are exhausted → `true`.
- If pattern still has segments but path is empty → only `**` segments are allowed.
- If first pattern segment is `"**"`:
  - If it's the only segment → `true`.
  - Otherwise, try matching the rest of the pattern against every suffix of the path.
- Otherwise, the first pattern segment must match the first path segment using `segment_matches` (handles `*` wildcards), and recursion continues on the rest.
- `segment_matches` handles `*` by trying to match the remainder of the pattern segment against suffixes of the path segment.

### Examples

```rust
use std::path::Path;
use librawssg_compiler::pattern::match_pattern;

assert!(match_pattern("**/*.html", Path::new("blog/post.html")));
assert!(match_pattern("blog/**/*.html", Path::new("blog/2024/post.html")));
assert!(match_pattern("*.html", Path::new("index.html")));
assert!(!match_pattern("*.html", Path::new("blog/post.html")));
assert!(match_pattern("**", Path::new("anything/at/all")));
assert!(match_pattern("**/*.md", Path::new("readme.md"))); // zero segments before .md
```

---

## Struct `Pipeline`

The `Pipeline` is the core execution engine. It is created by `PipelineBuilder::build()` and holds all necessary components.

```rust
pub struct Pipeline {
    config: Config,
    fs: Box<dyn FileSystem>,
    renderer: Box<dyn Renderer>,
    processors: Vec<Box<dyn Processor>>,
    context_builder: Box<dyn ContextBuilder>,
    generators: Vec<Box<dyn Generator>>,
    content_dir: PathBuf,
    output_dir: PathBuf,
}
```

All fields are private; access to configuration is provided via the `config()` method.

### Pipeline::config

```rust
#[must_use]
pub const fn config(&self) -> &Config
```

**Purpose**: Returns a reference to the configuration used by this pipeline.

**Returns**: `&Config`.

---

### Pipeline::run

```rust
pub fn run(&self) -> Result<()>
```

**Purpose**: Executes the full site generation process atomically.

**Behavior**:

1. Determines a temporary output directory: `output_dir.with_extension("tmp")`. For example, if `output_dir` is `"dist"`, the temp dir is `"dist.tmp"`.
2. If the temp dir already exists, it is removed.
3. Creates the temp dir.
4. Calls internal `generate_to(&tmp_dir)` to perform the actual generation into the temporary location.
5. If the final output directory exists, it is removed.
6. Attempts to rename the temp dir to the final output dir.
   - On success, returns `Ok(())`.
   - If rename fails with `CrossesDevices` error (different filesystems), fallback: copy the temp dir contents to the final output dir using `copy_dir_all` (internal method), then remove the temp dir.
   - For any other error, returns `Error::Generation("atomic rename failed: ...")`.

**Returns**: `Ok(())` on success, or `Err(Error)`.

**Note**: This atomic approach ensures that the final output directory is never left in a partially generated state; either the old output remains untouched (if generation fails) or the new output replaces it atomically (or near‑atomically).

---

### Pipeline Internal Workflow

The internal method `generate_to(output_base: &Path)` orchestrates the entire generation. The following steps are performed (not public, but described for understanding):

1. **Create output directory** – `fs.create_dir_all(output_base)`.
2. **Process documents** – Calls `process_documents()` to get a vector of `Document`.
3. **Group documents by content type** – Builds a `HashMap<String, Vec<Document>>`.
4. **Render non‑list documents** – For each `Document` where `is_list == false`, calls `render_document(output_base, doc)`.
5. **Generate list pages** – For each content type that has a matching `ContentRule` with `list_enabled == true`, `list_template` set, and at least one document, creates a synthetic list document (`is_list = true`) and renders it using the list template. The list document’s `list_items` are all documents of that content type. The output path is `"{content_type}/index.html"`.
6. **Copy static assets** – If the directory specified by `config.build.static_dir` exists, its entire contents are copied to `output_base/static_dir_name` (using `copy_dir_all`).
7. **Run generators** – Iterates over all `generators` and calls `generate()` for each, passing the output base.

#### Document Processing

`process_documents()` walks the content directory (using `fs.walk_dir`). For each file, it:

- Computes the path relative to `content_dir`.
- Iterates through the processors in order; the first processor whose `can_process(rel, &file_path)` returns `true` is used.
- Calls that processor’s `process(&*fs, rel, &content_dir)`, which returns `Option<Document>`.
- If a document is returned, its `content_type` is overridden based on the matching content rule (via `determine_content_type`), and its `depth` is set to the number of path components minus 1.
- The document is added to the result list.

If no processor matches, the file is ignored. If a processor returns `Ok(None)`, the file is skipped. If any processor returns an error, the whole processing fails.

#### Content Type Determination

`determine_content_type(rel)` iterates through the `config.content_rules` in **reverse order** (so later rules take precedence) and returns the `name` of the first rule whose `pattern` matches the relative path. If no rule matches, the content type defaults to `"page"`.

#### Rendering

`render_document` calls `template_for_document(doc)` to get the template name:

- Iterates through content rules, finds the rule whose `name` equals `doc.content_type`.
- If `doc.is_list` and the rule has a `list_template`, that template is used.
- Otherwise, the rule’s `template` is used.
- If no rule is found, returns `Error::Generation("no content rule found for type '...'")`.

The actual rendering uses:

- `context_builder.build_context(&config, doc)` to get a `RenderContext`.
- `renderer.render(template, &*ctx)` to produce the HTML string.
- `write_output(output_base, doc, html.as_bytes())` to write the result.

#### List Generation

When generating list pages, the pipeline creates a `Metadata` with `title = content_type` and empty description. Then constructs a `Document` with:

- `body`: empty string (the list template is expected to use `page_list_items`).
- `url`: `"{content_type}/index.html"`.
- `output_path`: `"{content_type}/index.html"`.
- `source_path`: `PathBuf::from("__list__")` (placeholder).
- `depth`: `1`.
- `content_type`: same as the content type.
- `is_list`: `true`.

Then sets `list_items` to the cloned vector of documents of that type, and renders using the list template.

#### Static Assets

The static directory from `config.build.static_dir` is copied verbatim. The destination is `output_base` joined with the static directory’s base name (e.g., if `static_dir = "static"`, files are copied to `output_base/static/`). Directories are recursively copied.

#### Generators

After all documents and static files are in place, each registered `Generator` is invoked with `&self` and `output_base`. This allows adding custom files like RSS feeds, sitemaps, etc.

---

## Error Handling

`librawssg_compiler` uses `librawssg_error::Error` for all fallible operations. The common error variants encountered:

- `Error::Config` – For configuration issues (e.g., missing renderer, invalid config file).
- `Error::Validation` – From `Config::validate`.
- `Error::Generation` – For errors during pipeline execution (e.g., write failures, unsafe output path).
- `Error::Io` – From filesystem operations (though these may be wrapped in `Error::Generation` in some places).
- `Error::Render` – From template rendering.

Methods return `Result<T>` (alias for `std::result::Result<T, librawssg_error::Error>`).

---

## Complete Example from Tests

The test file `full_compiler_test.rs` demonstrates a full working pipeline with mock components. Below is a simplified but complete example.

### Example Setup

Define mock renderer, context, context builder, and processor:

```rust
use librawssg_compiler::{PipelineBuilder, TeraContextBuilder};
use librawssg_config::{Config, ContentRule};
use librawssg_fs::{FileSystem, RealFs};
use librawssg_handler::{Document, Metadata, Processor};
use librawssg_templates::{RenderContext, Renderer};
use std::path::{Path, PathBuf};

// Mock renderer: returns "rendered:{template_name}"
struct MockRenderer;
impl Renderer for MockRenderer {
    fn render(&self, template_name: &str, _ctx: &dyn RenderContext) -> Result<String> {
        Ok(format!("rendered:{template_name}"))
    }
}

// Mock context
struct MockContext;
impl RenderContext for MockContext {
    fn as_any(&self) -> &dyn Any { self }
    fn as_mut_any(&mut self) -> &mut dyn Any { self }
}

// Mock context builder
struct MockContextBuilder;
impl ContextBuilder for MockContextBuilder {
    fn build_context(&self, _config: &Config, _doc: &Document) -> Result<Box<dyn RenderContext>> {
        Ok(Box::new(MockContext))
    }
}

// Processor that handles .html files
struct RawHtmlProcessor;
impl Processor for RawHtmlProcessor {
    fn name(&self) -> &'static str { "raw-html" }
    fn can_process(&self, relative_path: &Path, _original_path: &Path) -> bool {
        relative_path.extension().is_some_and(|ext| ext == "html")
    }
    fn process(&self, fs: &dyn FileSystem, relative_path: &Path, content_dir: &Path) -> Result<Option<Document>> {
        let full_path = content_dir.join(relative_path);
        let content = fs.read_to_string(&full_path)?;
        let url = relative_path.with_extension("html").to_string_lossy().to_string();
        let output_path = PathBuf::from(&url);
        let metadata = Metadata::new("Test", "Description")?;
        let doc = Document::new(
            metadata,
            content,
            url,
            output_path,
            relative_path.to_path_buf(),
            0,
            "page".to_string(),
            false,
        )?;
        Ok(Some(doc))
    }
}

// Config with one rule
fn setup_config() -> Config {
    let mut config = Config::new().with_site_name("Compiler Test");
    config.add_content_rule(ContentRule::new("page", "**/*.html", "base"));
    config
}

// Build pipeline
fn build_pipeline(content_dir: &Path, output_dir: &Path) -> Result<Pipeline> {
    PipelineBuilder::new()
        .config(setup_config())
        .content_dir(content_dir)
        .output_dir(output_dir)
        .with_fs(Box::new(RealFs))
        .with_renderer(Box::new(MockRenderer))
        .with_context_builder(Box::new(MockContextBuilder))
        .add_processor(Box::new(RawHtmlProcessor))
        .build()
}
```

### Running the Pipeline

```rust
let tmp = tempfile::TempDir::new()?;
let content_dir = tmp.path().join("content");
let output_dir = tmp.path().join("dist");
std::fs::create_dir_all(&content_dir)?;
std::fs::write(content_dir.join("index.html"), "<h1>Home</h1>")?;

let pipeline = build_pipeline(&content_dir, &output_dir)?;
pipeline.run()?;
```

### Verifying Output

```rust
let output_file = output_dir.join("index.html");
assert!(output_file.exists());
let content = std::fs::read_to_string(output_file)?;
assert_eq!(content, "rendered:base");
```

---

## Testing Suite Overview

The test file `tests/full_compiler_test.rs` contains comprehensive integration tests covering:

- Generation of a single page.
- Content type rules (different templates for different content types).
- List page generation.
- Static asset copying.
- Custom generators.
- Atomic replacement of existing output directory.
- Handling empty content directory.
- Skipping files that no processor handles.
- Error cases: missing renderer, missing context builder, invalid config.

Each test uses temporary directories (`tempfile`) and mock implementations to isolate components. The tests serve as executable examples of how to configure and run the pipeline.

---

## Conclusion

`librawssg_compiler` is the central execution engine of the static site generator. It provides a flexible builder to assemble the necessary components, a robust `Pipeline` that orchestrates all steps, and extension points via `Processor`, `Renderer`, `ContextBuilder`, and `Generator`. The pattern matching function and atomic output generation ensure correctness and safety. The comprehensive test suite demonstrates practical usage and edge cases.

For further details, refer to the source code and the test file.
