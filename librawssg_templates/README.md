# librawssg_templates

**Version**: 1.0.0 (implied)  
**Crate name**: `librawssg_templates`  
**Description**: Defines rendering abstractions for static site generators. Provides the `Renderer` and `RenderContext` traits, and an optional `TeraRenderer` implementation (when the `tera` feature is enabled) that integrates the Tera template engine.

---

## Table of Contents

1. [Overview](#overview)
2. [Modules and Features](#modules-and-features)
3. [Core Traits](#core-traits)
   - [`RenderContext`](#trait-rendercontext)
     - [Required Methods](#rendercontext-required-methods)
   - [`Renderer`](#trait-renderer)
     - [Required Method](#renderer-required-method)
4. [`TeraRenderer`](#struct-terarenderer)
   - [Struct Definition](#struct-definition)
   - [Constructor `new()`](#terarenderer-new)
   - [Method `add_raw_template()`](#terarenderer-add_raw_template)
   - [Method `add_template_file()`](#terarenderer-add_template_file)
   - [Method `add_template_files_from_dir()`](#terarenderer-add_template_files_from_dir)
   - [Method `load_templates_dir()`](#terarenderer-load_templates_dir)
   - [Method `enable_autoescape()`](#terarenderer-enable_autoescape)
   - [Method `render_str()`](#terarenderer-render_str)
   - [Method `as_tera()`](#terarenderer-as_tera)
   - [Method `as_tera_mut()`](#terarenderer-as_tera_mut)
   - [Trait Implementations](#terarenderer-trait-implementations)
     - [`Default`](#terarenderer-default)
     - [`Renderer` for `TeraRenderer`](#terarenderer-renderer-impl)
     - [`RenderContext` for `tera::Context`](#rendercontext-for-teracontext)
5. [Internal Helper Function](#internal-helper-function)
6. [Error Handling](#error-handling)
7. [Feature Gating](#feature-gating)
8. [Examples from Tests](#examples-from-tests)
   - [Basic Rendering](#basic-rendering)
   - [Loops, Filters, Conditions](#loops-filters-conditions)
   - [File Loading](#file-loading)
   - [Autoescaping](#autoescaping)
   - [Template Inheritance and Macros](#template-inheritance-and-macros)
   - [Context Downcasting](#context-downcasting)
9. [Testing Suite Overview](#testing-suite-overview)
10. [Conclusion](#conclusion)

---

## Overview

`librawssg_templates` provides a pluggable template rendering system. It abstracts the rendering process with two traits:

- **`Renderer`** – Defines the `render` method that takes a template name and a context, returning a rendered string.
- **`RenderContext`** – An object‑safe trait that allows type erasure for context objects; specifically, it provides `as_any` and `as_mut_any` to downcast to concrete context types.

The crate optionally includes a **`TeraRenderer`** implementation for the [Tera](https://tera.netlify.app/) template engine. This implementation is gated behind the `tera` feature flag.

---

## Modules and Features

The crate root (`lib.rs`) declares:

```rust
pub mod renderer;
#[cfg(feature = "tera")]
pub mod tera_renderer;

pub use renderer::{RenderContext, Renderer};
#[cfg(feature = "tera")]
pub use tera_renderer::TeraRenderer;
```

- **`renderer`** – Always available; contains the two core traits.
- **`tera_renderer`** – Only compiled when the `tera` feature is enabled; contains `TeraRenderer`.
- Re‑exports at the crate root make the traits and `TeraRenderer` easy to import.

The `tera` feature must be explicitly enabled in `Cargo.toml` to use `TeraRenderer`. Without it, the crate still provides the traits for custom renderer implementations.

---

## Core Traits

### Trait `RenderContext`

```rust
pub trait RenderContext: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
```

**Purpose**: Allows arbitrary context types to be passed to a `Renderer` as a trait object. The renderer can then downcast the `&dyn RenderContext` to the concrete context type it expects (e.g., `tera::Context`). This provides flexibility without requiring all renderers to accept a single concrete type.

**Requirements**:

- Implementors must be `Send + Sync` (thread‑safe).
- Must provide `as_any` and `as_mut_any` to expose the underlying `Any` reference.

**Typical Implementation**:

For any type `T`, you can implement:

```rust
impl RenderContext for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
```

**Example** (from tests):

```rust
struct MockContext;

impl RenderContext for MockContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
```

---

### Trait `Renderer`

```rust
pub trait Renderer: Send + Sync {
    fn render(&self, template_name: &str, context: &dyn RenderContext) -> Result<String>;
}
```

**Purpose**: Defines the rendering interface. A `Renderer` takes a template identifier (name) and a context object, and returns the rendered output as a `String`.

**Parameters**:

- `template_name`: A string identifying the template (e.g., `"index.html"`, `"blog/post.tera"`).
- `context`: A reference to an object implementing `RenderContext`. The renderer is expected to downcast this to the appropriate concrete context type.

**Returns**:

- `Ok(String)` containing the rendered output.
- `Err(librawssg_error::Error)` if rendering fails (e.g., template not found, invalid syntax, missing variable, or context type mismatch).

**Note**: Implementors must be `Send + Sync`.

**Example** (custom mock renderer):

```rust
struct MockRenderer { output: String }

impl Renderer for MockRenderer {
    fn render(&self, _template_name: &str, _context: &dyn RenderContext) -> Result<String> {
        Ok(self.output.clone())
    }
}
```

---

## Struct `TeraRenderer`

`TeraRenderer` is a wrapper around `tera::Tera`, providing convenient methods to load templates and render them using the `Renderer` trait. It is only available when the `tera` feature is enabled.

### Struct Definition

```rust
#[derive(Debug)]
pub struct TeraRenderer {
    tera: tera::Tera,
}
```

The `tera` field is private; access is provided via `as_tera` and `as_tera_mut`.

---

### `TeraRenderer::new`

```rust
#[must_use]
pub fn new() -> Self
```

**Purpose**: Creates a new `TeraRenderer` with an empty Tera instance (`tera::Tera::default()`).

**Returns**: A new `TeraRenderer`.

**Example**:

```rust
let renderer = TeraRenderer::new();
```

---

### `TeraRenderer::add_raw_template`

```rust
pub fn add_raw_template(&mut self, name: &str, content: &str) -> Result<()>
```

**Purpose**: Adds a template from a string, associating it with the given `name`. The template is parsed and stored internally.

**Parameters**:

- `name`: The template name (e.g., `"index.html"`, `"partial"`).
- `content`: The raw template source (e.g., `"Hello {{ name }}"`).

**Returns**:

- `Ok(())` if the template was added successfully.
- `Err(Error::Render)` if the template syntax is invalid (the underlying `tera::Error` is converted to a string and wrapped).

**Behavior**: Calls `tera.add_raw_template(name, content)`. Template names must be unique; adding a duplicate name will replace the existing template.

**Example**:

```rust
renderer.add_raw_template("hello", "Hello {{ name }}")?;
```

---

### `TeraRenderer::add_template_file`

```rust
pub fn add_template_file(&mut self, path: &Path) -> Result<()>
```

**Purpose**: Reads a template file from disk and adds it to the renderer. The template name is derived from the file name (including extension).

**Parameters**:

- `path`: Path to the template file.

**Returns**:

- `Ok(())` on success.
- `Err(Error::Io)` if the file cannot be read (wrapped as `Error::Io` with a message containing the original I/O error).
- `Err(Error::Render)` if the file name is not valid UTF‑8 or missing (unlikely).

**Behavior**:

1. Reads the file content using `std::fs::read_to_string`. On failure, maps to `Error::Io(std::io::Error::other(format!("{e}")))`.
2. Extracts the file name (the last component of the path) and converts it to a `&str`. If missing or non‑UTF‑8, returns `Error::Render("template file has no valid file name")`.
3. Calls `add_raw_template` with that file name as the template name.

**Example**:

```rust
renderer.add_template_file(Path::new("templates/index.html"))?;
// Template is registered as "index.html"
```

---

### `TeraRenderer::add_template_files_from_dir`

```rust
pub fn add_template_files_from_dir(&mut self, dir: &Path) -> Result<()>
```

**Purpose**: Adds all files directly inside a directory as templates. This method is **not recursive**; it only considers files in the immediate directory.

**Parameters**:

- `dir`: Directory containing template files.

**Returns**:

- `Ok(())` if at least the directory is readable and processing completes.
- `Err(Error::Io)` on directory read failure.
- `Err(Error::Render)` if any individual file cannot be added.

**Behavior**:

1. Reads the directory entries using `std::fs::read_dir`.
2. For each entry:
   - If the entry is a file, calls `add_template_file` with its path.
   - If that returns an error, the method immediately returns the error (fail‑fast).
3. Non‑file entries (subdirectories, symlinks) are ignored.

**Note**: Template names are the file names (including extensions).

**Example**:

```rust
renderer.add_template_files_from_dir(Path::new("templates/"))?;
// Adds all files in templates/ as templates with names like "base.tera", "index.html", etc.
```

---

### `TeraRenderer::load_templates_dir`

```rust
pub fn load_templates_dir(&mut self, dir: &Path) -> Result<()>
```

**Purpose**: Recursively loads all template files from a directory tree. Template names are derived from the relative path (using forward slashes as separators), allowing nested template structures (e.g., `"sub/nested.tera"`).

**Parameters**:

- `dir`: Root directory to traverse.

**Returns**:

- `Ok(())` on success.
- `Err(Error::Io)` for filesystem errors during traversal or reading.
- `Err(Error::Render)` for invalid UTF‑8 paths or component issues.

**Behavior**:

1. Canonicalizes the input directory (using `dir.canonicalize()`) to ensure a stable base.
2. Walks the directory recursively using `walkdir::WalkDir`. Only files are processed.
3. For each file:
   - Computes its path relative to the canonical directory using `strip_prefix`.
   - Converts the relative path to a template name using the internal helper `rel_path_to_template_name` (which joins components with `/` and rejects non‑normal components).
   - Reads the file content.
   - Calls `add_raw_template` with the computed template name and content.

**Note**: This method is similar to `add_template_files_from_dir` but recursive and with namespace‑like template names.

**Example**:

```rust
renderer.load_templates_dir(Path::new("templates"))?;
// If templates contains sub/child.tera, it can be referenced as "sub/child.tera"
```

---

### `TeraRenderer::enable_autoescape`

```rust
pub fn enable_autoescape(&mut self)
```

**Purpose**: Turns on automatic escaping for HTML, HTM, and XML file extensions. This is a convenience method that calls `tera.autoescape_on(vec!["html", "htm", "xml"])`.

**Parameters**: None.

**Returns**: Nothing.

**Behavior**: After calling this, templates with names ending in `.html`, `.htm`, or `.xml` will automatically escape variable output (HTML escaping). For other file extensions, autoescaping remains off.

**Example**:

```rust
renderer.enable_autoescape();
renderer.add_raw_template("page.html", "{{ user_input }}")?;
// Rendering will escape HTML special characters in user_input
```

---

### `TeraRenderer::render_str`

```rust
pub fn render_str(&self, template_str: &str, context: &dyn RenderContext) -> Result<String>
```

**Purpose**: Renders a one‑off template string without registering it. This is useful for small, inline templates.

**Parameters**:

- `template_str`: The template source as a string.
- `context`: A `&dyn RenderContext` that must downcast to `tera::Context`.

**Returns**:

- `Ok(String)` with rendered output.
- `Err(Error::Render)` if the context is not a `tera::Context` or if rendering fails.

**Behavior**:

1. Attempts to downcast `context.as_any()` to `&tera::Context`. If the cast fails, returns `Error::Render("invalid context type for Tera")`.
2. Calls `tera::Tera::one_off(template_str, tera_ctx, true)`. The third argument `true` enables autoescaping for the one‑off render **by default**, regardless of the renderer's autoescape settings.

**Note**: `render_str` always autoescapes (the `true` parameter forces autoescape). This may differ from `render` which respects the renderer's autoescape configuration.

**Example**:

```rust
let renderer = TeraRenderer::new();
let mut ctx = tera::Context::new();
ctx.insert("content", "<b>bold</b>");
let output = renderer.render_str("{{ content }}", &ctx)?;
// Output is escaped: "&lt;b&gt;bold&lt;&#x2F;b&gt;"
```

---

### `TeraRenderer::as_tera`

```rust
#[must_use]
pub const fn as_tera(&self) -> &tera::Tera
```

**Purpose**: Returns an immutable reference to the underlying `tera::Tera` instance. This allows advanced operations not directly exposed by `TeraRenderer`.

**Returns**: `&tera::Tera`.

**Example**:

```rust
let tera = renderer.as_tera();
// e.g., inspect registered templates
```

---

### `TeraRenderer::as_tera_mut`

```rust
#[must_use]
pub const fn as_tera_mut(&mut self) -> &mut tera::Tera
```

**Purpose**: Returns a mutable reference to the underlying `tera::Tera` instance. Useful for direct manipulation, such as adding templates or changing settings.

**Returns**: `&mut tera::Tera`.

**Example**:

```rust
let tera_mut = renderer.as_tera_mut();
tera_mut.add_raw_template("direct", "Hello")?;
```

---

### Trait Implementations

#### `TeraRenderer::Default`

```rust
impl Default for TeraRenderer {
    fn default() -> Self {
        Self::new()
    }
}
```

Allows creating a `TeraRenderer` with `TeraRenderer::default()`, equivalent to `new()`.

#### `Renderer` for `TeraRenderer`

```rust
impl Renderer for TeraRenderer {
    fn render(&self, template_name: &str, context: &dyn RenderContext) -> Result<String> {
        let tera_ctx = context
            .as_any()
            .downcast_ref::<tera::Context>()
            .ok_or_else(|| Error::Render("invalid context type for Tera".into()))?;
        self.tera
            .render(template_name, tera_ctx)
            .map_err(|e| Error::Render(e.to_string()))
    }
}
```

- **`render` method**:
  - Downcasts the context to `tera::Context`.
  - Delegates to `tera.render(template_name, tera_ctx)`.
  - Maps any error to `Error::Render`.

#### `RenderContext` for `tera::Context`

```rust
impl RenderContext for tera::Context {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
        self
    }
}
```

This implementation allows `tera::Context` to be used directly as a `RenderContext` when calling `render`. Users typically create a `tera::Context`, populate it, and pass `&ctx` to `render`.

---

## Internal Helper Function

`rel_path_to_template_name` (private)

```rust
fn rel_path_to_template_name(rel_path: &Path) -> Result<String>
```

**Purpose**: Converts a relative path (from `strip_prefix`) into a template name string using forward slashes as separators. It rejects paths with unusual components (prefixes, root, parent, or current directory).

**Parameters**:

- `rel_path`: A relative path (assumed to have been stripped of a base).

**Returns**:

- `Ok(String)` with the template name (e.g., `"sub/nested.tera"`).
- `Err(Error::Render)` if:
  - A component is not `Normal` (e.g., contains `..` or `/` absolute parts).
  - The path contains non‑UTF‑8 characters.
  - The resulting name is empty.

**Note**: This function is not public but is essential for `load_templates_dir`.

---

## Error Handling

All fallible methods in `TeraRenderer` return `librawssg_error::Result<T>`. The errors originate from:

- Filesystem operations → converted to `Error::Io` (with `std::io::Error::other` wrapper to preserve the original message).
- Tera template parsing/rendering → converted to `Error::Render` with the error message as string.
- Context type mismatch → `Error::Render("invalid context type for Tera")`.
- Invalid template file name or path component → `Error::Render`.

The `Renderer` trait method also returns `Result<String>`, allowing custom renderers to use the same error type.

---

## Feature Gating

- The core traits (`Renderer`, `RenderContext`) are always available.
- `TeraRenderer` and the `tera` integration are only compiled when the `tera` feature is enabled.
- Tests that use `TeraRenderer` are also gated with `#![cfg(feature = "tera")]`.

To enable the feature, add to `Cargo.toml`:

```toml
[dependencies]
librawssg_templates = { version = "...", features = ["tera"] }
```

---

## Examples from Tests

The test suite (`tera_tests.rs` and `unit_tests.rs`) provides extensive examples. Below are selected snippets with explanations.

### Basic Rendering

```rust
let mut renderer = TeraRenderer::new();
renderer.add_raw_template("simple", "{{ title }}")?;

let mut ctx = tera::Context::new();
ctx.insert("title", "Hello World");
let output = renderer.render("simple", &ctx)?;
assert_eq!(output, "Hello World");
```

### Loops, Filters, Conditions

```rust
renderer.add_raw_template("loop", "{% for item in items %}{{ item }}{% if not loop.last %},{% endif %}{% endfor %}")?;
// context: items = ["a","b","c"] -> output "a,b,c"

renderer.add_raw_template("filter", "{{ title | upper }}")?;
// context: title = "Hello" -> "HELLO"

renderer.add_raw_template("condition", "{% if number > 40 %}high{% else %}low{% endif %}")?;
// context: number = 42 -> "high"
```

### File Loading

```rust
// Add a single file
let file_path = dir.path().join("hello.tera");
std::fs::write(&file_path, "{{ name }}")?;
renderer.add_template_file(&file_path)?;
// Template name is "hello.tera"

// Add all files from a directory (non-recursive)
renderer.add_template_files_from_dir(dir.path())?;

// Recursive loading with namespaced names
renderer.load_templates_dir(dir.path())?;
// If sub/nested.tera exists, use renderer.render("sub/nested.tera", &ctx)
```

### Autoescaping

```rust
renderer.enable_autoescape();
renderer.add_raw_template("esc.html", "{{ content }}")?;
let mut ctx = tera::Context::new();
ctx.insert("content", "<script>alert(1)</script>");
let output = renderer.render("esc.html", &ctx)?;
// Output: &lt;script&gt;alert(1)&lt;&#x2F;script&gt;
```

Note: `render_str` always autoescapes:

```rust
let output = renderer.render_str("{{ content }}", &ctx)?;
// Also escaped
```

### Template Inheritance and Macros

```rust
renderer.add_raw_template("base", "<html>{% block content %}Default{% endblock %}</html>")?;
renderer.add_raw_template("child", "{% extends \"base\" %}{% block content %}Child content{% endblock %}")?;
// Rendering "child" yields "<html>Child content</html>"

renderer.add_raw_template("macro", "{% macro hello(name) %}Hello, {{ name }}{% endmacro hello %}{{ self::hello(name=\"World\") }}")?;
// Rendering "macro" yields "Hello, World"
```

### Context Downcasting

```rust
let ctx = sample_context();
let dyn_ctx: &dyn RenderContext = &ctx;
assert!(dyn_ctx.as_any().is::<tera::Context>());
```

This shows how the `RenderContext` trait enables type erasure and safe downcasting.

---

## Testing Suite Overview

The crate contains two test files:

- **`unit_tests.rs`** (always compiled): Tests the core traits using mock implementations, verifies re‑exports are available, and ensures the traits can be used without the `tera` feature.
- **`tera_tests.rs`** (compiled only with `tera` feature): Comprehensive tests for `TeraRenderer` including:
  - Basic variable substitution, loops, filters, conditions.
  - Error cases (missing template, missing variable, invalid syntax).
  - Loading templates from files and directories (both non‑recursive and recursive).
  - Autoescaping behavior.
  - Template inheritance, macros, includes.
  - Context downcasting.
  - Access to underlying `Tera` via `as_tera` and `as_tera_mut`.

The tests use `tempfile` for temporary directories and `walkdir` for directory traversal validation.

---

## Conclusion

`librawssg_templates` offers a flexible and extensible template rendering abstraction. The core `Renderer` and `RenderContext` traits allow any template engine to be integrated, while the built‑in `TeraRenderer` provides a powerful, full‑featured implementation for the Tera engine. With methods for loading templates from files or directories, autoescaping control, and direct access to the underlying engine, it covers the needs of most static site generators.

The crate is designed with testability and thread‑safety in mind, and the comprehensive test suite serves as both documentation and validation. By enabling the `tera` feature, developers can immediately start rendering templates with minimal setup.
