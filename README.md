# librawssg

A modular static site generator library for Rust.

`librawssg` is a collection of crates that together form a flexible and extensible framework for building static site generators. The project is designed with modularity, testability, and safety in mind, leveraging Rust's type system and trait abstractions.

## Features

- **Modular architecture** – Each aspect (configuration, filesystem, content processing, templating, compilation) is isolated into its own crate.
- **Pluggable processors** – Define custom content processors via the `Processor` trait.
- **Template engine integration** – Built-in support for Tera templates through the `TeraRenderer` (optional, enabled by default).
- **Strong filesystem abstraction** – Trait-based filesystem with built-in path traversal protection.
- **Atomic output generation** – The build pipeline writes to a temporary directory and atomically replaces the final output.
- **Comprehensive configuration** – YAML/JSON support, validation, and nested site/build settings.
- **Extensible** – Add custom renderers, context builders, and post-processing generators.
- **Strict linting** – Deny-level lints for clippy and rustc ensure high code quality.
- **Demo application** – A complete example showing how to assemble the parts into a working static site.

## Repository Structure

The workspace consists of the following crates:

| Crate                 | Description                                                    |
| --------------------- | -------------------------------------------------------------- |
| `librawssg`           | Facade crate that re-exports all other crates for convenience. |
| `librawssg_config`    | Configuration data structures and validation.                  |
| `librawssg_fs`        | Filesystem abstraction trait and real implementation.          |
| `librawssg_handler`   | Core document, metadata, and processor contracts.              |
| `librawssg_templates` | Rendering traits and Tera implementation.                      |
| `librawssg_compiler`  | Build pipeline orchestration.                                  |
| `librawssg_error`     | Unified error types and result alias.                          |
| `librawssg_demo`      | Example application demonstrating usage of the framework.      |

## Getting Started

### Prerequisites

- Rust toolchain (stable, edition 2024) – install via [rustup](https://rustup.rs/)
- Cargo (comes with Rust)

### Building the Project

Clone the repository and build all crates:

```bash
git clone https://github.com/mroczect/librawssg.git
cd librawssg
cargo build
```

### Running the Demo

The `librawssg_demo` crate provides a working example. To run it:

```bash
cargo run -p librawssg_demo
```

This will process `.raw` HTML fragment files from `librawssg_demo/src/content`, render them using a Tera template, copy static assets, and output the site into `librawssg_demo/dist`.

### Using as a Library

Add `librawssg` to your `Cargo.toml`:

```toml
[dependencies]
librawssg = "1.0.0"
```

Then you can import the necessary components. Here is a minimal example that sets up a pipeline:

```rust
use librawssg::{
    Config, ContentRule, Document, FileSystem, Metadata, PipelineBuilder, Processor,
    RealFs, RenderContext, Renderer, TeraContextBuilder, TeraRenderer,
};
use std::path::{Path, PathBuf};

// Implement a custom processor for .txt files
struct TextProcessor;
impl Processor for TextProcessor {
    fn name(&self) -> &'static str { "text" }
    fn can_process(&self, rel: &Path, _orig: &Path) -> bool {
        rel.extension().and_then(|e| e.to_str()) == Some("txt")
    }
    fn process(
        &self,
        fs: &dyn FileSystem,
        rel: &Path,
        content_dir: &Path,
    ) -> librawssg::Result<Option<Document>> {
        let body = fs.read_to_string(&content_dir.join(rel))?;
        let meta = Metadata::new("Page", "Description")?;
        let url = rel.with_extension("html").to_string_lossy().to_string();
        let doc = Document::new(
            meta,
            body,
            url.clone(),
            PathBuf::from(&url),
            rel.to_path_buf(),
            0,
            "page".to_string(),
            false,
        )?;
        Ok(Some(doc))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new().with_site_name("My Site");
    config.add_content_rule(ContentRule::new("page", "**/*.txt", "base.tera"));
    config.build.content_dir = "content".into();
    config.build.output_dir = "dist".into();
    config.build.static_dir = "static".into();

    let mut renderer = TeraRenderer::new();
    renderer.load_templates_dir(Path::new("templates"))?;

    let pipeline = PipelineBuilder::new()
        .config(config)
        .content_dir("content")
        .output_dir("dist")
        .with_fs(Box::new(RealFs))
        .with_renderer(Box::new(renderer))
        .with_context_builder(Box::new(TeraContextBuilder))
        .add_processor(Box::new(TextProcessor))
        .build()?;

    pipeline.run()?;
    println!("Site generated!");
    Ok(())
}
```

For a more detailed example, see the `librawssg_demo` source code.

## Core Concepts

### Configuration

The `Config` struct holds all settings required for the build. It includes:

- `site`: Site-wide metadata (name, description, navigation, etc.)
- `build`: Paths for content, output, templates, and static assets.
- `content_rules`: A list of `ContentRule` objects that map file patterns to templates.
- `extra`: Arbitrary key-value data.

Configuration can be loaded from YAML or JSON using `Config::from_yaml_str` / `Config::from_json_str`.

### Content Processing

Content files are processed by implementations of the `Processor` trait. Each processor declares which files it can handle via `can_process()`, and then transforms them into `Document` objects. The pipeline walks the content directory, determines the appropriate processor for each file, and collects the resulting documents.

### Rendering

The `Renderer` trait abstracts template rendering. The built-in `TeraRenderer` uses the Tera template engine. A `ContextBuilder` creates the render context for each document; the default `TeraContextBuilder` populates it with page and site data.

### Build Pipeline

The `PipelineBuilder` assembles all components (filesystem, renderer, processors, context builder, generators) and produces a `Pipeline`. Calling `pipeline.run()` performs the following steps:

1. Processes all content files.
2. Renders non-list documents.
3. Optionally generates list pages (index pages) for content types with list support enabled.
4. Copies static assets.
5. Executes any custom generators.
6. Atomically replaces the output directory.

## Customization

You can extend the framework by implementing the following traits:

- **`Processor`** – For handling new file types or custom transformations.
- **`Renderer`** and **`RenderContext`** – To integrate a different template engine.
- **`ContextBuilder`** – To customize the data passed to templates.
- **`Generator`** – To add extra outputs like RSS feeds, sitemaps, or search indexes.

All components are passed to the pipeline as boxed trait objects, so they are easily swappable.

## Development

### Workspace Lints

The workspace enforces strict linting via `[workspace.lints]` in the root `Cargo.toml`. Many clippy and rustc lints are set to `deny`, including `unsafe_code = "forbid"`, `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`, and many others. This ensures high code quality and safety. When contributing, please ensure your code passes `cargo clippy --all --all-targets --all-features -- -D warnings` and `cargo fmt --check`.

### Running Tests

```bash
cargo test --workspace
```

### Formatting

```bash
cargo fmt --all
```

## Contributing

Contributions are welcome! Please read `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md` for guidelines. By participating, you agree to abide by the project's code of conduct.

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

## Acknowledgements

This project uses the following open-source crates (among others):

- [Tera](https://github.com/Keats/tera) – Template engine
- [Serde](https://serde.rs/) – Serialization framework
- [WalkDir](https://github.com/BurntSushi/walkdir) – Directory traversal
