# librawssg_demo

A complete demo application for **librawssg**, a modular static site generator framework written in Rust. This project showcases how to assemble the various `librawssg` crates into a working static site generator that reads raw HTML fragments, renders them through a Tera template, copies static assets, and outputs a fully static website.

---

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Project Structure](#project-structure)
4. [Prerequisites](#prerequisites)
5. [Installation & Build](#installation--build)
6. [Running the Demo](#running-the-demo)
7. [How It Works](#how-it-works)
   - [The `RawFileProcessor`](#the-rawfileprocessor)
   - [Template Rendering with Tera](#template-rendering-with-tera)
   - [Configuration](#configuration)
   - [Static Assets](#static-assets)
   - [Output Generation](#output-generation)
8. [Customization Guide](#customization-guide)
   - [Adding New Content Files](#adding-new-content-files)
   - [Changing the Template](#changing-the-template)
   - [Adding Custom Processors](#adding-custom-processors)
   - [Adding Generators](#adding-generators)
9. [Underlying Crates](#underlying-crates)
10. [Troubleshooting](#troubleshooting)
11. [License](#license)

---

## Overview

`librawssg_demo` demonstrates a minimal but functional static site generator built with the `librawssg` framework. It uses:

- **`librawssg_config`** for configuration management.
- **`librawssg_handler`** to define a custom `Processor` that handles `.raw` files.
- **`librawssg_templates`** for Tera-based rendering.
- **`librawssg_fs`** for filesystem abstraction (using `RealFs`).
- **`librawssg_compiler`** to orchestrate the build pipeline.

The demo processes `.raw` files (containing HTML fragments) from `src/content`, renders them using a Tera template (`base.tera`), copies static files from `src/static`, and outputs the final site into the `dist/` folder.

---

## Features

- **Modular architecture**: Each component (processing, rendering, file I/O, configuration) is separated and replaceable.
- **Custom content processing**: The included `RawFileProcessor` reads `.raw` files and converts them into `Document` objects.
- **Template rendering**: Uses the Tera template engine with a `TeraContextBuilder` to inject page data.
- **Static asset copying**: Automatically copies the `static` directory to the output.
- **Atomic output**: The build process writes to a temporary directory and atomically replaces the final output.
- **Extensible**: Easily add new processors, renderers, context builders, or generators.

---

## Project Structure

```
librawssg_demo/
├── Cargo.toml
└── src/
    ├── content/
    │   ├── about.raw
    │   └── index.raw
    ├── static/
    │   └── style.css
    ├── templates/
    │   └── base.tera
    └── main.rs
```

- **`Cargo.toml`** – Defines the package and dependencies on the `librawssg_*` crates (via path).
- **`src/main.rs`** – Entry point; configures and runs the pipeline.
- **`src/content/`** – Contains source content files (`.raw`). These are processed by `RawFileProcessor`.
- **`src/static/`** – Static assets (CSS, images, etc.) that are copied verbatim to the output.
- **`src/templates/`** – Tera templates used for rendering.
- **`dist/`** – Generated output (created at runtime, not stored in version control).

---

## Prerequisites

- **Rust toolchain** (stable, edition 2024) – Install via [rustup](https://rustup.rs/).
- **Cargo** – Comes with Rust.
- The `librawssg_*` crates must be available at the relative paths specified in `Cargo.toml`. This demo assumes a workspace layout where the crates are siblings of `librawssg_demo`.

---

## Installation & Build

1. **Clone the repository** (or navigate to the demo directory inside the workspace).

2. **Build the project**:

   ```bash
   cargo build
   ```

3. **Run the demo**:
   ```bash
   cargo run
   ```

Upon successful execution, the output site will be generated in the `dist/` folder (relative to the project root).

---

## Running the Demo

Execute:

```bash
cargo run
```

The program will:

1. Read the configuration (created programmatically in `main.rs`).
2. Load templates from `src/templates`.
3. Process all `.raw` files in `src/content`.
4. Render each document using the `base.tera` template.
5. Copy static files from `src/static`.
6. Write everything into `dist/` atomically.

After completion, you can open `dist/index.html` in a browser to view the generated site.

---

## How It Works

### The `RawFileProcessor`

The `RawFileProcessor` implements the `Processor` trait from `librawssg_handler`. Its responsibilities:

- **`can_process`**: Returns `true` only for files with a `.raw` extension.
- **`process`**:
  - Reads the file content using the provided `FileSystem`.
  - Derives a title from the file stem (e.g., `my-page` becomes `My Page`).
  - Creates a `Metadata` object with the title.
  - Constructs a `Document` with:
    - `body`: the raw HTML content (will be inserted into the template).
    - `url`: the same relative path but with `.html` extension.
    - `output_path`: the relative output path (same as URL).
    - `content_type`: hardcoded to `"page"`.

### Template Rendering with Tera

- **Renderer**: `TeraRenderer` (from `librawssg_templates`) is used. It loads all templates from the `src/templates` directory recursively.
- **Context Builder**: `TeraContextBuilder` creates a `tera::Context` containing:
  - `site`: the full `SiteConfig`.
  - `page_title`, `page_description`, `page_author`, etc.
  - `page_content`: the raw body of the document (inserted with `| safe` filter in the template).
- **Template**: `base.tera` defines the overall HTML structure. It uses `{{ page_title }}` and `{{ page_content | safe }}` to inject data.

### Configuration

In `main.rs`, a `Config` object is built:

- `site_name` is set to `"Demo Site"`.
- One `ContentRule` is added: `name = "page"`, `pattern = "**/*.raw"`, `template = "base.tera"`.
- Build paths (`content_dir`, `output_dir`, `static_dir`) are set to absolute paths under the project root.

### Static Assets

The `config.build.static_dir` points to `src/static`. During generation, the pipeline copies this directory recursively to the output. In this demo, the `style.css` file will appear at `dist/static/style.css`. The template references it via `/style.css` assuming the static directory is copied with its name preserved (i.e., `static/`). To make the CSS load correctly, the static directory is copied as `dist/static/`, and the link in the template should be `/static/style.css`. However, the provided template uses `/style.css` – this might be a slight mismatch. In a real deployment you may want to adjust either the static directory name or the link. The demo as provided will copy `static/` into `dist/static/`, but the HTML link expects `/style.css` which would 404 unless the server serves the static directory at root. For local file viewing, it won't work directly. This is a common issue; you can either change the static dir to be copied to the root (by setting `static_dir` to a path with no subdirectory) or modify the template link to `/static/style.css`. The demo code keeps the default behavior; user may need to adjust.

### Output Generation

The `Pipeline` orchestrates the build:

1. Processes all content files.
2. Groups documents by content type.
3. Renders non-list documents.
4. (No list pages in this demo because `list_enabled` is not set.)
5. Copies static assets.
6. Runs any custom generators (none in this demo).
7. Atomically replaces the `dist/` directory.

---

## Customization Guide

### Adding New Content Files

Simply create a new `.raw` file in `src/content/`. For example, `src/content/contact.raw`:

```html
<h1>Contact</h1>
<p>Email us at hello@example.com</p>
```

The next `cargo run` will automatically process it and generate `dist/contact.html`.

### Changing the Template

Edit `src/templates/base.tera`. You can use any Tera syntax. The available context variables include:

- `page_title`
- `page_content` (raw HTML)
- `page_description`
- `page_author`
- `page_date`
- `page_tags`
- `page_url`
- `page_depth`
- `page_type`
- `page_is_list`
- `page_list_items`
- `site` (the full `SiteConfig`)

You can also add custom fields to `Metadata` or use the `extra` maps.

### Adding Custom Processors

Implement the `Processor` trait and add it to the pipeline builder with `.add_processor(Box::new(MyProcessor))`. For example, you could create a Markdown processor that handles `.md` files.

### Adding Generators

Implement the `Generator` trait and add it with `.add_generator(Box::new(MyGenerator))`. Generators run after documents and static files are written, allowing you to create RSS feeds, sitemaps, or search indexes.

---

## Underlying Crates

This demo relies on the following `librawssg` crates (all located in sibling directories):

- [`librawssg_compiler`](../librawssg_compiler) – Pipeline and builder.
- [`librawssg_config`](../librawssg_config) – Configuration types.
- [`librawssg_fs`](../librawssg_fs) – Filesystem abstraction.
- [`librawssg_handler`](../librawssg_handler) – `Document`, `Metadata`, and `Processor` trait.
- [`librawssg_templates`](../librawssg_templates) – Rendering traits and Tera implementation.
- [`librawssg_error`](../librawssg_error) – Unified error types.

All are used via path dependencies, so they must be present in the workspace.

---

## Troubleshooting

- **Build errors about missing crates**  
  Ensure all `librawssg_*` crates are checked out at the correct relative paths (siblings to this demo). Check `Cargo.toml` for the `path` attributes.

- **Static CSS not loading**  
  By default, the static directory is copied with its name (`static/`). The template currently links to `/style.css`. Either:
  - Change the template link to `/static/style.css`, or
  - Modify `config.build.static_dir` to a path whose basename is empty (e.g., copy contents directly to output root) by adjusting the pipeline's static copying logic (not recommended for this demo).

- **`cargo run` fails with permission errors**  
  The output directory `dist/` may already exist and be locked. Try deleting it manually or ensuring you have write permissions.

- **Tera template syntax errors**  
  Validate your template syntax. The error messages from Tera are descriptive and will indicate the problem line.

---

## License

This demo is licensed under the **MIT License**. See the `LICENSE` file in the repository root for details.
