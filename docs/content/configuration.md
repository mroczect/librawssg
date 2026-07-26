---
title: Configuration
desc: Complete YAML configuration reference for librawssg
---

librawssg is configured through a single YAML file. Every aspect of the site – metadata, directory layout, content types, and generators – is defined here.

The library ships with sensible defaults, so you only need to specify what changes from the default behaviour.

---

## Loading Configuration

There are two ways to provide configuration:

### From a YAML file (recommended)

```rust
use librawssg::site::builder::SiteBuilder;

let site = SiteBuilder::new()
    .load_config("config.yml")?
    .with_template_renderer(Box::new(tera))
    .with_markdown_renderer(Box::new(md))
    .build()?;
```

`load_config` reads, parses, and **validates** the file before returning. Any error (missing file, invalid YAML, failed validation) is returned immediately.

### Programmatically

```rust
use librawssg::types::RawssgConfig;

let mut config = RawssgConfig::default();
config.site.site_name = "My Site".into();
config.build.content_dir = "pages".into();
config.content_types.push(ContentTypeDef {
    name: "page".into(),
    pattern: "**/*.md".into(),
    template: "base.html".into(),
    list_template: None,
    list_enabled: false,
});

let site = SiteBuilder::new()
    .config(config)
    .with_template_renderer(Box::new(tera))
    .with_markdown_renderer(Box::new(md))
    .build()?;
```

Use this approach when you need to generate configuration dynamically or don't want a separate YAML file.

---

## Validation

When you call `load_config` or `build`, the configuration is automatically validated. The following rules are enforced:

| Rule                                                                           | Error message                                         |
| ------------------------------------------------------------------------------ | ----------------------------------------------------- |
| `site.site_name` must not be empty                                             | `"site.name cannot be empty"`                         |
| At least one `content_type` must be defined                                    | `"at least one content_type must be defined"`         |
| Every content type must have a non‑empty `name`                                | `"content_type name cannot be empty"`                 |
| Every content type must have a non‑empty `template`                            | `"content_type '...' must have a template"`           |
| Every content type `pattern` must be a valid glob                              | `"Invalid glob pattern '...'"`                        |
| If RSS is enabled, `rss.template` and `rss.path` must be non‑empty             | `"rss.template is required when RSS enabled"`         |
| If sitemap is enabled, `sitemap.template` and `sitemap.path` must be non‑empty | `"sitemap.template is required when sitemap enabled"` |

Validation happens **before** any file processing, saving you from errors later in the build.

---

## Full YAML Structure

```yaml
site: # Site-wide metadata (available in templates as `site.xxx`)
  site_name: # Required. Must not be empty.
  description: # Optional.
  language: # Optional. Default: "en"
  base_url: # Optional. Used in RSS/sitemap.
  author: # Optional.
  repo_url: # Optional.
  license: # Optional.
  navbar: # Optional. List of NavItem.
  sidebar: # Optional. List of NavItem.

build: # Directory layout
  content_dir: # Default: "content"
  output_dir: # Default: "dist"
  templates_dir: # Default: "templates"
  static_dir: # Default: "static"

content_types: # List of content type definitions (at least one required)
  - name: # Required. Identifier (e.g. "blog", "page")
    pattern: # Required. Glob pattern (e.g. "blog/*.md")
    template: # Required. Template name (e.g. "post.html")
    list_template: # Optional. Template for list pages
    list_enabled: # Optional. Default: false

generators: # Automatic RSS and sitemap generation
  rss:
    enabled: # Default: true
    path: # Required if enabled. Output path (e.g. "rss.xml")
    template: # Required if enabled. Template name (e.g. "rss.xml")
  sitemap:
    enabled: # Default: true
    path: # Required if enabled. Output path (e.g. "sitemap.xml")
    template: # Required if enabled. Template name (e.g. "sitemap.xml")
```

---

## `site` – Site Metadata

All fields under `site` are accessible in templates under the `site` object. For example, `site.site_name`, `site.author`, `site.navbar`.

### Fields

| Field         | Type     | Required | Default    | Description                                    |
| ------------- | -------- | -------- | ---------- | ---------------------------------------------- |
| `site_name`   | `string` | **Yes**  | `"rawssg"` | Name of the site. Used in `<title>`, RSS, etc. |
| `description` | `string` | No       | `None`     | Short description. Used in meta tags and RSS.  |
| `language`    | `string` | No       | `"en"`     | HTML language code (e.g. `"id"`, `"fr"`).      |
| `base_url`    | `string` | No       | `None`     | Full public URL. Used in RSS and sitemap.      |
| `author`      | `string` | No       | `None`     | Author name. Available as `site.author`.       |
| `repo_url`    | `string` | No       | `None`     | Repository URL. Available as `site.repo_url`.  |
| `license`     | `string` | No       | `None`     | License name. Available as `site.license`.     |
| `navbar`      | `list`   | No       | `[]`       | Navigation bar items. See [NavItem](#navitem). |
| `sidebar`     | `list`   | No       | `[]`       | Sidebar items. See [NavItem](#navitem).        |

### Example

```yaml
site:
  site_name: "My Awesome Blog"
  description: "Thoughts about Rust and static sites"
  language: "en"
  base_url: "https://example.com"
  author: "Jane Doe"
  repo_url: "https://github.com/janedoe/my-blog"
  license: "MIT"
  navbar:
    - label: "Home"
      url: "/"
    - label: "About"
      url: "/about.html"
    - label: "Blog"
      url: "/blog/index.html"
  sidebar:
    - label: "RSS"
      url: "/rss.xml"
    - label: "GitHub"
      url: "https://github.com/janedoe/my-blog"
```

### NavItem

| Field   | Type     | Description                        |
| ------- | -------- | ---------------------------------- |
| `label` | `string` | Display text for the link          |
| `url`   | `string` | Link target (relative or absolute) |

---

## `build` – Directory Layout

Controls where the library looks for input files and where it writes output.

### Fields

| Field           | Type     | Default       | Description                                                |
| --------------- | -------- | ------------- | ---------------------------------------------------------- |
| `content_dir`   | `string` | `"content"`   | Directory containing Markdown files                        |
| `output_dir`    | `string` | `"dist"`      | Directory where generated HTML is written                  |
| `templates_dir` | `string` | `"templates"` | Directory containing template files (if loading from disk) |
| `static_dir`    | `string` | `"static"`    | Directory with static assets (copied as-is)                |

### Example

```yaml
build:
  content_dir: "my-content"
  output_dir: "public"
  templates_dir: "my-templates"
  static_dir: "my-static"
```

### Notes

- All paths are relative to the **current working directory** when the binary runs.
- The `templates_dir` is **not used** by the library directly. You must load templates yourself (see the demo or docs site for an example). The field is kept for reference and future use.
- The `static_dir` is processed automatically: every file inside is copied to the output directory.
- Non‑Markdown files in `content_dir` (images, CSS, JS) are also copied automatically.

---

## `content_types` – Content Type Definitions

This is the heart of the content pipeline. Every Markdown file must be matched by at least one content type.

### Fields

| Field           | Type     | Required | Default | Description                                                                                  |
| --------------- | -------- | -------- | ------- | -------------------------------------------------------------------------------------------- |
| `name`          | `string` | **Yes**  | –       | Identifier for this type (e.g. `"blog"`, `"page"`). Used as `content_type` in `PageContext`. |
| `pattern`       | `string` | **Yes**  | –       | Glob pattern relative to `content_dir`. Supports `*` (one segment) and `**` (any segments).  |
| `template`      | `string` | **Yes**  | –       | Name of the template used for single pages of this type.                                     |
| `list_template` | `string` | No       | `None`  | Template for list pages (e.g. `blog/index.html`).                                            |
| `list_enabled`  | `bool`   | No       | `false` | If `true`, a list page is generated automatically.                                           |

### How patterns work

| Pattern        | Matches                             | Does not match                  |
| -------------- | ----------------------------------- | ------------------------------- |
| `*.md`         | `index.md`, `about.md`              | `blog/post.md`                  |
| `blog/*.md`    | `blog/post.md`                      | `blog/2024/post.md`, `index.md` |
| `blog/**/*.md` | `blog/post.md`, `blog/2024/post.md` | `index.md`                      |
| `**/*.md`      | Everything ending in `.md`          | –                               |

Patterns are relative to `content_dir`. They use a custom glob engine that supports `*` and `**` wildcards.

### How list pages work

When `list_enabled` is `true` and `list_template` is set:

1. All pages of that type are collected.
2. A `PageContext` with `is_list = true` is created.
3. The list of items is available in the template as the `pages` variable.

Example template (`blog_list.html`):

```html
{% extends "base.html" %} {% block content %}
<h2>All Posts</h2>
<ul>
  {% for post in pages %}
  <li>
    <a href="{{ base_path }}{{ post.url }}">{{ post.frontmatter.title }}</a>
    <span>({{ post.pub_date }})</span>
  </li>
  {% endfor %}
</ul>
{% endblock %}
```

The list page is generated at `{name}/index.html` (e.g. `blog/index.html`).

### Example

```yaml
content_types:
  - name: "page"
    pattern: "*.md"
    template: "base.html"
    # No list for regular pages

  - name: "blog"
    pattern: "blog/*.md"
    template: "blog_post.html"
    list_template: "blog_list.html"
    list_enabled: true

  - name: "tutorial"
    pattern: "tutorials/**/*.md"
    template: "tutorial.html"
    list_template: "tutorial_list.html"
    list_enabled: true
```

---

## `generators` – RSS and Sitemap

Automatically generate RSS and sitemap files for your site.

### RSS

| Field      | Type     | Default | Description                                             |
| ---------- | -------- | ------- | ------------------------------------------------------- |
| `enabled`  | `bool`   | `true`  | Whether to generate an RSS feed                         |
| `path`     | `string` | –       | Output path relative to `output_dir` (e.g. `"rss.xml"`) |
| `template` | `string` | –       | Template name for the RSS XML                           |

The RSS feed collects all pages with `content_type == "blog"` (sorted by date, newest first) and passes them as the `posts` variable to the template.

Example RSS template:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
    <channel>
        <title>{{ site.site_name }}</title>
        <description>{{ site.description }}</description>
        <link>{{ base_url }}</link>
        {% for post in posts %}
        <item>
            <title>{{ post.frontmatter.title }}</title>
            <link>{{ base_url }}/{{ post.url }}</link>
            <description>{{ post.frontmatter.desc }}</description>
            <pubDate>{{ post.pub_date }}</pubDate>
        </item>
        {% endfor %}
    </channel>
</rss>
```

### Sitemap

| Field      | Type     | Default | Description                                                 |
| ---------- | -------- | ------- | ----------------------------------------------------------- |
| `enabled`  | `bool`   | `true`  | Whether to generate a sitemap                               |
| `path`     | `string` | –       | Output path relative to `output_dir` (e.g. `"sitemap.xml"`) |
| `template` | `string` | –       | Template name for the sitemap XML                           |

All pages (including list pages) are passed as the `pages` variable.

Example sitemap template:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{% for page in pages %}
    <url>
        <loc>{{ base_url }}/{{ page.url }}</loc>
        {% if page.pub_date %}
        <lastmod>{{ page.pub_date }}</lastmod>
        {% endif %}
    </url>
{% endfor %}
</urlset>
```

### Example configuration

```yaml
generators:
  rss:
    enabled: true
    path: "feed.xml"
    template: "rss.xml"
  sitemap:
    enabled: true
    path: "sitemap.xml"
    template: "sitemap.xml"
```

To disable a generator entirely:

```yaml
generators:
  rss:
    enabled: false
  sitemap:
    enabled: false
```

---

## Complete Example

Here is a full, annotated configuration file that you can use as a starting point:

```yaml
# =============================================================================
# librawssg Configuration
# =============================================================================

# Site metadata – available in templates as `site.xxx`
site:
  site_name: "My Site"
  description: "A blog about Rust"
  language: "en"
  base_url: "https://example.com"
  author: "Your Name"
  repo_url: "https://github.com/you/your-site"
  license: "MIT"

  navbar:
    - label: "Home"
      url: "/"
    - label: "About"
      url: "/about.html"
    - label: "Blog"
      url: "/blog/index.html"

  sidebar:
    - label: "RSS"
      url: "/rss.xml"

# Directory paths (all relative to working directory)
build:
  content_dir: "content"
  output_dir: "dist"
  templates_dir: "templates"
  static_dir: "static"

# Content type definitions
content_types:
  - name: "page"
    pattern: "*.md"
    template: "base.html"

  - name: "blog"
    pattern: "blog/*.md"
    template: "blog_post.html"
    list_template: "blog_list.html"
    list_enabled: true

# RSS and Sitemap
generators:
  rss:
    enabled: true
    path: "rss.xml"
    template: "rss.xml"
  sitemap:
    enabled: true
    path: "sitemap.xml"
    template: "sitemap.xml"
```

Save this as `config.yml` in your project root, then load it with `SiteBuilder::new().load_config("config.yml")`.
