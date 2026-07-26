---
title: Contributing
desc: How to contribute to librawssg
---

Thank you for your interest in contributing to **librawssg**.  
We welcome contributions of all kinds: bug reports, feature requests, documentation improvements, code patches, and design discussions.

This guide outlines the entire contribution process so that everyone can work together effectively.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Building and Testing](#building-and-testing)
- [Coding Style](#coding-style)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)
- [Documentation](#documentation)
- [Community](#community)
- [Recognition](#recognition)

---

## Code of Conduct

This project adheres to a minimal set of social rules:

- **Be respectful** – treat others as you would like to be treated.
- **Be constructive** – focus on the issue, not the person.
- **Be inclusive** – welcome people of all backgrounds and experience levels.

Harassment, discrimination, or hostile behaviour is not tolerated.  
If you experience or witness such conduct, please contact the maintainers immediately.

Read the full [Code of Conduct](code_of_conduct.html).

---

## Ways to Contribute

You don't have to write code to make a difference. Here are some ways you can help:

| Contribution type    | Impact                                              |
| -------------------- | --------------------------------------------------- |
| **Bug reports**      | Help us find and fix problems                       |
| **Feature requests** | Shape the future of the library                     |
| **Documentation**    | Make the library easier to learn                    |
| **Code patches**     | Fix bugs, add features, improve performance         |
| **Code review**      | Help maintain quality by reviewing PRs              |
| **Testing**          | Write tests, report edge cases                      |
| **Spread the word**  | Write blog posts, give talks, share on social media |

---

## Getting Started

### 1. Fork the repository

Click the **Fork** button at the top of [github.com/mroczect/librawssg](https://github.com/mroczect/librawssg).

### 2. Clone your fork

```bash
git clone https://github.com/YOUR_USERNAME/librawssg.git
cd librawssg
```

### 3. Add the upstream remote

This keeps your fork in sync with the main repository.

```bash
git remote add upstream https://github.com/mroczect/librawssg.git
```

### 4. Create a branch

Use a descriptive name that reflects what you're working on.

```bash
git checkout -b feat/my-awesome-feature
```

Branch naming conventions:

| Prefix      | Purpose            | Example                |
| ----------- | ------------------ | ---------------------- |
| `feat/`     | New feature        | `feat/custom-handlers` |
| `fix/`      | Bug fix            | `fix/path-traversal`   |
| `docs/`     | Documentation      | `docs/api-reference`   |
| `test/`     | Adding tests       | `test/site-builder`    |
| `refactor/` | Code restructuring | `refactor/error-types` |
| `chore/`    | Maintenance tasks  | `chore/update-deps`    |

### 5. Keep your branch up to date

```bash
git fetch upstream
git rebase upstream/master
```

Rebase instead of merge to keep the history clean.  
If you're uncomfortable with rebasing, merging is also acceptable.

---

## Development Environment

### Required tools

- **Rust** – Install via [rustup](https://rustup.rs). The latest stable version is recommended.
  ```bash
  rustup update stable
  ```
- **Cargo** – Comes with Rust. No additional setup needed.
- **Git** – For version control.

### Recommended tools

- **cargo-edit** – Easily manage dependencies from the command line.
  ```bash
  cargo install cargo-edit
  ```
- **cargo-watch** – Automatically rebuild and test on file changes.
  ```bash
  cargo install cargo-watch
  ```
- **cargo-audit** – Check for security vulnerabilities in dependencies.
  ```bash
  cargo install cargo-audit
  ```

### Editor setup

Any text editor works, but we recommend one with Rust support:

- **VS Code** with the `rust-analyzer` extension
- **IntelliJ IDEA** with the Rust plugin
- **Helix** or **Neovim** with built-in LSP support

Configure your editor to:

- Run `rustfmt` on save
- Show Clippy warnings inline
- Enable `rust-analyzer` for autocompletion and type hints

---

## Project Structure

Understanding the codebase helps you make better contributions.

```
librawssg/
├── src/                     # Library source code
│   ├── config/              # Configuration loading (YAML, defaults)
│   │   ├── mod.rs           # ConfigLoader trait
│   │   └── loader.rs        # YamlConfigLoader implementation
│   ├── error.rs             # RawssgError enum (thiserror + miette)
│   ├── frontmatter.rs       # YAML frontmatter extraction
│   ├── fs/                  # Filesystem abstraction
│   │   ├── mod.rs           # FileSystem trait
│   │   └── real.rs          # RealFs implementation
│   ├── markdown.rs          # MarkdownRenderer trait (+ optional pulldown)
│   ├── serve/               # Optional dev server
│   │   ├── mod.rs           # Static file server
│   │   └── watcher.rs       # File watcher (notify)
│   ├── site/                # Core site building logic
│   │   ├── mod.rs           # TemplateRenderer trait, ContentHandler trait
│   │   ├── builder.rs       # SiteBuilder and Site
│   │   ├── page.rs          # Page context building
│   │   ├── feed.rs          # RSS feed generation
│   │   └── sitemap.rs       # Sitemap generation
│   ├── types.rs             # All data types (config, frontmatter, context)
│   ├── util.rs              # Path safety, slugify, glob matching
│   └── lib.rs               # Crate root, re-exports
├── tests/                   # Integration tests
│   ├── common/              # Mock implementations (MockFs, MockRenderers)
│   │   └── mod.rs
│   ├── config_tests.rs
│   ├── error_tests.rs
│   ├── frontmatter_tests.rs
│   ├── fs_mock_tests.rs
│   ├── fs_real_tests.rs
│   ├── integration_test.rs
│   ├── markdown_tests.rs
│   ├── site_builder_tests.rs
│   ├── site_page_tests.rs
│   └── util_tests.rs
├── demo/                    # Full working example
├── docs/                    # Documentation site source
└── Cargo.toml               # Project manifest
```

### Key design principles

1. **Everything is a trait** – Filesystem, rendering, content processing are all behind traits. This makes the library testable and extensible.
2. **Minimal dependencies** – Only essential crates are required. Everything else is optional.
3. **Safety first** – Path traversal is prevented at every level. Output is atomic.
4. **Validate early** – Configuration errors are caught at build time, not generation time.

---

## Building and Testing

All commands are run from the repository root.

### Build

```bash
# Basic build (no optional features)
cargo build

# Build with all features
cargo build --all-features

# Release build (optimised)
cargo build --release
```

### Run tests

```bash
# Run all tests (only tests that don't require features)
cargo test

# Run tests with Tera and pulldown features
cargo test --features tera,pulldown

# Run tests with dev server feature
cargo test --features serve

# Run all tests with every feature
cargo test --all-features

# Run a specific test file
cargo test --test config_tests

# Run a specific test function
cargo test test_name
```

### Format and lint

```bash
# Check formatting (CI will fail if this isn't clean)
cargo fmt --all -- --check

# Auto-format all code
cargo fmt --all

# Run Clippy with strict warnings
cargo clippy --all-targets --all-features -- -D warnings
```

**These checks are enforced in CI.** Run them locally before pushing.

### Watch mode (auto-test on changes)

```bash
cargo watch -x "test --all-features" -x "clippy --all-targets --all-features -- -D warnings"
```

---

## Coding Style

We follow the standard Rust style guide with a few additional conventions.

### General

- Follow what `cargo fmt` produces. No arguments.
- All Clippy warnings are errors. If Clippy complains, fix it.
- Keep functions small – aim for under 50 lines.
- One responsibility per function.
- Use meaningful variable names.

### Error handling

- Use `Result<T, RawssgError>` for all fallible operations.
- Use the `?` operator liberally.
- Add context to errors where helpful:
  ```rust
  fs.read_to_string(path)
      .map_err(|e| RawssgError::Config(format!("cannot read config: {}", e)))?;
  ```
- Don't use `.unwrap()` or `.expect()` in library code unless you can prove the invariant.

### Documentation

- Every public item must have a `///` doc comment.
- Include a short description, not just a repetition of the name.
- Add a code example when the usage isn't obvious.

Example:

````rust
/// Loads a site configuration from a YAML file.
///
/// The file is validated after loading. Returns an error if the file
/// cannot be read or contains invalid YAML.
///
/// # Example
/// ```rust
/// let builder = SiteBuilder::new()
///     .load_config("config.yml")
///     .expect("failed to load config");
/// ```
pub fn load_config<P: AsRef<Path>>(mut self, path: P) -> Result<Self, RawssgError> {
    // ...
}
````

### Testing

- Write tests for all new functionality.
- Place unit tests in the same file as the code they test, inside a `#[cfg(test)]` module.
- Place integration tests in the `tests/` directory.
- Use the mock implementations in `tests/common/mod.rs` for testing without real files.

### Performance

- Don't optimise prematurely.
- If you're making a performance claim, include benchmarks.
- Use `cargo bench` if benchmarks are available.

---

## Commit Messages

We use the [Conventional Commits](https://www.conventionalcommits.org/) format.

### Format

```
type(scope): short description

Optional longer explanation of the change, including motivation
and any breaking changes.
```

### Types

| Type       | When to use                                           |
| ---------- | ----------------------------------------------------- |
| `feat`     | A new feature                                         |
| `fix`      | A bug fix                                             |
| `docs`     | Documentation changes                                 |
| `test`     | Adding or updating tests                              |
| `refactor` | Code changes that neither fix a bug nor add a feature |
| `chore`    | Maintenance tasks (dependencies, CI, etc.)            |
| `style`    | Formatting, missing semicolons, etc. (no code change) |
| `perf`     | Performance improvements                              |

### Scope

The scope is usually `librawssg`, but can be more specific:

| Scope       | When to use                 |
| ----------- | --------------------------- |
| `librawssg` | Changes to the core library |
| `ci`        | CI pipeline changes         |
| `docs`      | Documentation site changes  |
| `demo`      | Demo project changes        |

### Examples

```text
feat(librawssg): add support for custom content handlers

Introduce the ContentHandler trait that allows users to define their own
file processing logic. Includes default MarkdownPageHandler.

BREAKING CHANGE: SiteBuilder::new() no longer sets up default handlers.
```

```text
fix(librawssg): prevent path traversal in safe_path

safe_path now normalises paths before checking boundaries, preventing
attacks using ../ sequences on not-yet-existing files.
```

```text
docs(librawssg): add API reference for PageContext

Document all fields of PageContext with examples showing how they are
populated from Markdown frontmatter.
```

---

## Pull Request Process

### Before you submit

1. **Update your branch** – Rebase onto the latest `master`.

   ```bash
   git fetch upstream
   git rebase upstream/master
   ```

2. **Run all checks** – Make sure everything passes.

   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   ```

3. **Check your commits** – Squash trivial commits, ensure messages follow the convention.

### Submitting

1. Push your branch to your fork.
2. Open a pull request against `mroczect/librawssg:master`.
3. Fill out the PR template completely.

### PR description

Your PR description should answer these questions:

- **What** does this change do?
- **Why** is this change needed?
- **How** was it implemented (brief overview)?
- **Breaking changes** – List any changes that break existing code.
- **Testing** – Describe how you tested the change.
- **Related issues** – Link any issues this PR addresses.

Example:

```markdown
## What

Add a `ContentHandler` trait that allows users to define custom file
processing logic.

## Why

Currently, only Markdown files can be processed. Users with other file
types (AsciiDoc, reStructuredText) have no way to integrate them.

## How

- Defined the `ContentHandler` trait in `site/mod.rs`
- Added `add_handler()` method to `SiteBuilder`
- Converted existing Markdown handling into `MarkdownPageHandler`
- Added `StaticFileHandler` as a catch-all

## Breaking changes

- `SiteBuilder::new()` no longer sets up default handlers.
  Users must call `.add_handler()` if they remove defaults.

## Testing

- Added `test_custom_handler()` to `tests/site_builder_tests.rs`
- All existing tests pass with the updated builder

## Related issues

Closes #42
```

### Review process

1. A maintainer will review your code within a few days.
2. They may ask questions or request changes.
3. Make the requested changes in new commits, then push.
4. Once approved, the maintainer will merge your PR via **squash merge** to keep the history linear.

### After merge

- Delete your branch.
- Update your fork's `master` branch.

```bash
git checkout master
git pull upstream master
git push origin master
git branch -d feat/my-feature
```

---

## Reporting Bugs

Found a bug? We appreciate you taking the time to report it.

### Before reporting

1. **Search existing issues** – Someone might have already reported it.
2. **Check the version** – Make sure you're using the latest release.

### Writing a good bug report

Include as much of the following as possible:

```markdown
## Description

A clear and concise description of the bug.

## Steps to reproduce

1. Create a project with this configuration: ...
2. Add this content file: ...
3. Run `cargo run`
4. See error: ...

## Expected behaviour

The site should generate with ...

## Actual behaviour

Instead, the following error occurs: ...

## Environment

- OS: Ubuntu 24.04
- Rust version: rustc 1.80.0
- librawssg version: v0.3.0
- Features enabled: tera, pulldown

## Minimal reproduction

Link to a repository or paste minimal code that reproduces the issue.
```

The more detail you provide, the faster we can fix the bug.

---

## Feature Requests

Have an idea to make librawssg better? We want to hear it.

### Writing a good feature request

```markdown
## The problem

I want to use custom template engines, but currently I have to
implement TemplateRenderer which is not well documented.

## Proposed solution

Add a guide showing how to implement TemplateRenderer for a
custom engine, with Handlebars as an example.

## Alternatives considered

I could use Tera, but my existing templates are in Handlebars.

## Additional context

Handlebars is widely used in the JavaScript ecosystem, so
supporting it would attract more users.
```

### What makes a good feature request

- **Describes a real problem** – Not just "it would be cool if…"
- **Fits the project scope** – librawssg is a kernel, not a full website builder.
- **Is specific** – Vague requests are hard to act on.
- **Considers alternatives** – Shows you've thought about the problem.

For large features, consider opening a discussion issue first to gather feedback before writing code.

---

## Documentation

Documentation is just as important as code. We follow these principles:

### What to document

- Every public API item (types, traits, functions)
- Configuration options
- How to get started (tutorials)
- How to contribute (this guide)

### Where documentation lives

| Documentation type | Location                        |
| ------------------ | ------------------------------- |
| API docs           | `///` comments in source code   |
| User guide         | `docs/content/*.md` (this site) |
| README             | `README.md` in the root         |
| Release notes      | GitHub releases                 |

### Writing good documentation

- Write in plain English.
- Use short sentences and paragraphs.
- Include code examples that can be copied and run.
- Use consistent terminology.
- Update documentation when you change behaviour.

### Building the documentation site

```bash
cd docs
cargo run
```

The generated site appears in `docs/dist/`.

---

## Community

### Communication channels

- **GitHub Issues** – Bug reports and feature requests
- **GitHub Discussions** – Questions and general conversation
- **Pull Requests** – Code contributions and review

### Getting help

If you're stuck or have questions:

1. Check the [documentation](/).
2. Search [existing issues](https://github.com/mroczect/librawssg/issues).
3. Open a new discussion or issue if you can't find an answer.

### Etiquette

- Be patient – maintainers are volunteers.
- Be clear – the more context you provide, the better the answer.
- Be kind – everyone was a beginner once.

---

## Recognition

All contributors are recognised in the project. When your PR is merged:

- Your name appears in the Git history.
- You may be listed in the README (ask if you'd like to be added).
- You get the satisfaction of knowing you helped build something useful.

---

Thank you for contributing to **librawssg**. Your effort makes this project better for everyone.
