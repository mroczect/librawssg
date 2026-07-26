---
title: Installation
desc: Add librawssg to your Rust project in under a minute
---

librawssg is **not yet published on [crates.io](https://crates.io)**, but you can still use it today directly from the Git repository.  
All methods below take less than a minute – pick the one that fits your workflow.

---

## Prerequisites

- **Rust** `1.96` or later (install via [rustup](https://rustup.rs))
- A Cargo project with `edition = "2024"` (or `2021` if you prefer)

---

## Method 1: `cargo add` (Git dependency – recommended)

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

## Method 2: Manual `Cargo.toml` entry

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

## Method 4: Full clone and build

```bash
git clone https://github.com/mroczect/librawssg.git
cd librawssg
cargo build --release
```

Then reference it as a path dependency (see Method 3).

---

## Feature Flags

librawssg is modular – you only pay for what you use.

| Flag       | What it enables             | Adds crate(s)         |
| ---------- | --------------------------- | --------------------- |
| `tera`     | Built‑in `TeraRenderer`     | `tera`                |
| `pulldown` | Built‑in `PulldownMarkdown` | `pulldown-cmark`      |
| `serve`    | Dev server with live reload | `tiny_http`, `notify` |

All features are **disabled by default**.  
If you don't enable `tera` or `pulldown`, you must provide your own implementations of `TemplateRenderer` and `MarkdownRenderer`.

---

## Verify your installation

Create a small test binary to confirm everything compiles:

```rust
// src/main.rs
use librawssg::site::builder::SiteBuilder;

fn main() {
    let _ = SiteBuilder::new();
    println!("librawssg is ready!");
}
```

```bash
cargo run
```

If you see `"librawssg is ready!"`, the library is correctly linked.

---

## Minimal working example

Here's a complete, copy‑paste ready project that generates a real site.

**1. Create a new binary project**

```bash
cargo new my-site
cd my-site
```

**2. Add the dependency (choose one method)**

Using `cargo add`:

```bash
cargo add --git https://github.com/mroczect/librawssg.git --tag v0.4.0 librawssg --features tera,pulldown
```

Or edit `Cargo.toml` manually (see Method 2).

**3. Create folders and a content file**

```bash
mkdir content templates static
echo '---
title: Hello
desc: My first page
---
# Welcome!' > content/index.md

echo '<html><body>{{ page_content | safe }}</body></html>' > templates/base.html
```

**4. Create `config.yml`**

```yaml
site:
  site_name: "My Site"

build:
  content_dir: "content"
  output_dir: "dist"
  templates_dir: "templates"
  static_dir: "static"

content_types:
  - name: "page"
    pattern: "*.md"
    template: "base.html"
    list_enabled: false
```

**5. Replace `src/main.rs`**

```rust
use librawssg::markdown::PulldownMarkdown;
use librawssg::site::builder::SiteBuilder;
use librawssg::site::TeraRenderer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tera = TeraRenderer::new();
    let templates_dir = Path::new("templates");
    for entry in std::fs::read_dir(templates_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let content = std::fs::read_to_string(&path)?;
            tera.add_raw_template(&name, &content)?;
        }
    }

    let site = SiteBuilder::new()
        .load_config("config.yml")?
        .content_dir("content")
        .output_dir("dist")
        .with_template_renderer(Box::new(tera))
        .with_markdown_renderer(Box::new(PulldownMarkdown))
        .build()?;

    site.generate()?;
    println!("Site generated in dist/");
    Ok(())
}
```

**6. Run it**

```bash
cargo run
open dist/index.html
```

You should see a simple HTML page with "Welcome!" rendered from Markdown.

---

## Troubleshooting

### `error: failed to parse manifest` when using a tag

Make sure the tag exists on GitHub (e.g., `v0.4.0`). You can also use `branch = "master"` temporarily, but **prefer a tag for stability**.

### `cannot find trait implementation` for Tera or Pulldown

You forgot to enable the `tera` and `pulldown` features. Add them to your `Cargo.toml` or `cargo add` command.

### `No context implementation available` error at runtime

You tried to build a site without a template renderer. Make sure you call `.with_template_renderer(...)` before `.build()`.

### `File not found` when running from a different directory

The `content_dir`, `templates_dir`, and `static_dir` paths are relative to the current working directory. Run the binary from the project root, or use absolute paths.

---

## Next steps

- [Configuration](configuration.html) – Learn every YAML option
- [API Reference](references.html) – Discover all traits and types
- [Demo project](https://github.com/mroczect/librawssg/tree/master/demo) – A full‑featured example with blog, RSS, and sitemap