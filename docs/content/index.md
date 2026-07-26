---
title: librawssg
desc: A safety‑first, engine‑agnostic static site generator kernel for Rust
---

**librawssg** is the core engine that powers `rawssg`.  
It gives you complete freedom to build your own static site generator with the tools you already know.

**Engine‑agnostic** – Bring your own template engine (Tera, Handlebars, …) and Markdown renderer (pulldown‑cmark, comrak, …) through simple Rust traits.

**Safety‑first** – Built‑in path‑traversal protection, atomic output generation, and strict configuration validation mean you never have to worry about broken builds or security holes.

**Batteries optional** – The library ships with zero mandatory dependencies. Enable the built‑in Tera and pulldown‑cmark implementations with a single feature flag, or plug in your own.

---

## Why librawssg?

- **You control the pipeline** – Write your own `main.rs` and compose exactly the pieces you need.
- **Test everything** – Every component is abstracted behind a trait; mock the filesystem and renderers for fast, reliable tests.
- **No magic** – Configuration is validated up‑front. Errors are typed and described with `miette` diagnostics.

---

## Quick example

```rust
use librawssg::site::builder::SiteBuilder;
use librawssg::site::TeraRenderer;
use librawssg::markdown::PulldownMarkdown;

let site = SiteBuilder::new()
    .load_config("config.yml")?
    .with_template_renderer(Box::new(tera))
    .with_markdown_renderer(Box::new(md))
    .build()?;

site.generate()?;
```

---

## Explore the documentation

- [Installation](installation.html) – Add librawssg to your project
- [Configuration](configuration.html) – Full YAML reference
- [API Reference](references.html) – All public types and traits
- [Contributing](contributing.html) – Help us improve
