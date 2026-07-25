# Contributing to librawssg

Thank you for your interest in contributing to librawssg. This document outlines the process for reporting issues, proposing changes, and submitting code contributions. Following these guidelines helps maintain the quality and consistency of the project.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Building and Testing](#building-and-testing)
- [Coding Style](#coding-style)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)
- [Documentation](#documentation)
- [Community](#community)

---

## Code of Conduct

This project adheres to a minimal set of social rules: be respectful, constructive, and inclusive. Harassment, discrimination, or hostile behaviour is not tolerated. If you experience or witness such conduct, please contact the maintainers.

---

## Getting Started

1. **Fork the repository** on GitHub.
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/librawssg.git
   cd librawssg
   ```
3. **Add the upstream remote** to keep your fork in sync:
   ```bash
   git remote add upstream https://github.com/mroczect/librawssg.git
   ```
4. **Create a branch** for your work:
   ```bash
   git checkout -b feat/my-feature
   ```

---

## Development Environment

- **Rust**: Install the latest stable Rust toolchain via [rustup](https://rustup.rs/).
- **Dependencies**: The project uses several crates (serde, thiserror, miette, walkdir, chrono, etc.). They will be fetched automatically by Cargo. Optional features (`tera`, `pulldown`, `serve`) pull additional crates only when enabled.
- **OS support**: librawssg is a pure Rust library and should compile and run on all platforms supported by Rust (Linux, macOS, Windows). Ensure any changes remain cross-platform.

---

## Building and Testing

All commands below are run from the repository root.

### Build

```bash
cargo build
```

To build with all features enabled:

```bash
cargo build --all-features
```

### Run tests

```bash
cargo test
cargo test --features tera,pulldown
cargo test --all-features
```

This runs unit tests, integration tests (located in `tests/`), and doc-tests. All tests must pass before a pull request is accepted.

### Lint and format

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

These are enforced in CI. Run them locally to avoid surprises.

---

## Coding Style

- Follow the standard Rust formatting (enforced by `cargo fmt`).
- Use `rustc` and `clippy` lints strictly; any warning is treated as an error in CI.
- Write idiomatic Rust:
  - Use `Result` and `Option` appropriately.
  - Prefer `From` implementations for error conversions.
  - Document public API items with `///` comments.
- Keep functions small and focused.
- Add tests for new functionality.
- For any platform-specific code (unlikely in a pure SSG kernel), guard with `#[cfg(...)]` attributes.

---

## Commit Messages

Use [conventional commit](https://www.conventionalcommits.org/) format:

```
type(scope): short description

Optional longer explanation.
```

**Types**: `feat`, `fix`, `docs`, `test`, `ci`, `chore`, `refactor`, `style`.

**Scope**: `librawssg` (for core library), `ci`, `docs`, etc.

Examples:
- `feat(librawssg): add support for custom content handlers`
- `fix(librawssg): prevent path traversal when outputting files`
- `docs(librawssg): add API reference for PageContext`

This format enables automatic changelog generation and clear history.

---

## Pull Request Process

1. Ensure your branch is based on an up-to-date `master`.
2. Run `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets --all-features -- -D warnings` to verify there are no issues.
3. If you added or modified public API, update the README and any relevant documentation comments.
4. Push your branch and open a pull request against the `master` branch of the main repository.
5. In the PR description:
   - Explain what the change does and why.
   - Mention any breaking changes.
   - Link to any related issues.
   - Note if documentation updates are included.
6. The CI will run automatically. All checks must be green.
7. A maintainer will review your code. Please respond to feedback and make requested changes.
8. Once approved, the PR will be merged via squash merge to keep the history linear.

---

## Reporting Bugs

Open an issue on GitHub and include:

- A clear description of the problem.
- Steps to reproduce.
- Expected vs actual behaviour.
- Environment details: OS, Rust version (`rustc --version`), librawssg version or commit hash.
- If applicable, a minimal code example that demonstrates the bug.

---

## Feature Requests

Feature requests are welcome. When opening an issue:

- Describe the feature and the problem it solves.
- Explain how it fits into the library's scope.
- Be open to discussion about design and implementation.

For large features, consider opening an issue first to gather feedback before writing code.

---

## Documentation

- The main documentation is the README and API docs (`cargo doc`).
- If you add a new public type or function, include clear doc comments with examples where appropriate.
- Update the README if a new feature or major change affects the usage flow.

---

## Community

- The main communication channel is GitHub issues and pull requests.
- For questions or informal discussion, you can reach out via the repository's Discussions tab if enabled.

Thank you for contributing to librawssg. Your effort helps make the project better for everyone.