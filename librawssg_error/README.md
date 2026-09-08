# librawssg_error

**Version**: 1.0.0 (implied)  
**Crate name**: `librawssg_error`  
**Description**: Defines a comprehensive error enum and `Result` alias for use across the `librawssg` static site generator ecosystem. The error type is built with `thiserror` for ergonomic `Display` and `Error` implementations, supports source chaining, and is designed to cover all common failure modes in SSG operations.

---

## Table of Contents

1. [Overview](#overview)
2. [Dependencies](#dependencies)
3. [The `Error` Enum](#the-error-enum)
   - [Enum Definition](#enum-definition)
   - [Attributes and Derives](#attributes-and-derives)
   - [Non‑Exhaustive](#non-exhaustive)
4. [Variants](#variants)
   - [`Io`](#io)
   - [`Config`](#config)
   - [`Metadata`](#metadata)
   - [`Render`](#render)
   - [`Processor`](#processor)
   - [`Generator`](#generator)
   - [`PathTraversal`](#pathtraversal)
   - [`MissingConfig`](#missingconfig)
   - [`Generation`](#generation)
   - [`NotFound`](#notfound)
   - [`Serialization`](#serialization)
   - [`Validation`](#validation)
   - [`Duplicate`](#duplicate)
   - [`InvalidState`](#invalidstate)
   - [`Internal`](#internal)
5. [`Result<T>` Type Alias](#resultt-type-alias)
6. [Error Sources and `std::error::Error`](#error-sources-and-stderroerror)
7. [Conversion from `std::io::Error`](#conversion-from-stdioerror)
8. [Usage Examples from Tests](#usage-examples-from-tests)
   - [Display Messages](#display-messages)
   - [Using the `?` Operator](#using-the--operator)
   - [Source Chain](#source-chain)
   - [Property Tests](#property-tests)
9. [Guidelines for Error Usage](#guidelines-for-error-usage)
10. [Testing Suite Overview](#testing-suite-overview)
11. [Conclusion](#conclusion)

---

## Overview

The `librawssg_error` crate provides a single, unified error type for the entire static site generator ecosystem. Instead of having each module define its own error types, they all share this `Error` enum, which categorizes failures into well‑defined variants. The enum is derived with `thiserror::Error`, giving each variant an automatic `Display` implementation based on a custom message pattern, and an automatic `std::error::Error` implementation that preserves source chains when applicable.

The crate also exports a `Result<T>` type alias, simplifying function signatures throughout the codebase.

Key features:

- **Rich error categories** – 15 distinct variants covering I/O, configuration, parsing, rendering, processing, generation, security, and internal errors.
- **Source chaining** – The `Metadata` variant can wrap an underlying error (e.g., a YAML parsing error) and expose it via `source()`.
- **Convenient conversion** – `From<std::io::Error>` allows using the `?` operator directly in functions returning `Result<T, Error>`.
- **Non‑exhaustive** – The enum is marked `#[non_exhaustive]`, enabling future additions without breaking downstream code.

---

## Dependencies

- `thiserror` – Provides the `#[derive(Error)]` macro that generates `Display` and `Error` implementations from the attributes.
- `core::error::Error` (or `std::error::Error`) – Used as a trait object for the `source` field in the `Metadata` variant.

No other external crates are required.

---

## The `Error` Enum

### Enum Definition

```rust
use core::error::Error as CoreError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Failed to parse metadata in {path}")]
    Metadata {
        path: PathBuf,
        #[source]
        source: Box<dyn CoreError + Send + Sync>,
    },

    #[error("Template rendering error: {0}")]
    Render(String),

    #[error("Content processor error: {0}")]
    Processor(String),

    #[error("Generator error: {0}")]
    Generator(String),

    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),

    #[error("Missing configuration key: {0}")]
    MissingConfig(String),

    #[error("Site generation error: {0}")]
    Generation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate value: {0}")]
    Duplicate(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

### Attributes and Derives

- **`#[derive(Debug, Error)]`** – Derives `Debug` and `std::error::Error`. The `Error` derive from `thiserror` also generates a `Display` implementation based on the `#[error("...")]` attributes.
- **`#[non_exhaustive]`** – Indicates that the enum may gain new variants in future releases. Downstream crates must not exhaustively match on this enum; they must include a wildcard arm (`_`) when matching.

### Non‑Exhaustive

Because the enum is non‑exhaustive, external code cannot write:

```rust
match err {
    Error::Io(_) => ...,
    Error::Config(_) => ...,
    // all variants...
}
```

without including a catch‑all arm:

```rust
match err {
    Error::Io(_) => ...,
    Error::Config(_) => ...,
    // ...
    _ => { /* handle unknown future variants */ }
}
```

This ensures forward compatibility.

---

## Variants

### `Io`

```rust
#[error("I/O error: {0}")]
Io(#[from] std::io::Error),
```

- **Description**: Wraps a standard library I/O error. Used for any filesystem operation failure (reading, writing, deleting, etc.).
- **Fields**: Contains a single `std::io::Error`.
- **Display**: `"I/O error: {underlying_io_error_message}"`.
- **`#[from]`**: Automatically provides `From<std::io::Error> for Error`, allowing the `?` operator in functions returning `Result<T, Error>`.
- **Source**: `source()` returns `Some(&io_error)` because `std::io::Error` implements `std::error::Error`.

**Example**:

```rust
let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
let err = Error::Io(io_err);
assert_eq!(err.to_string(), "I/O error: file missing");
```

---

### `Config`

```rust
#[error("Configuration error: {0}")]
Config(String),
```

- **Description**: Indicates a problem with configuration data (e.g., invalid YAML, malformed settings).
- **Fields**: A `String` containing a human‑readable description.
- **Display**: `"Configuration error: {message}"`.
- **Source**: `None` (no underlying error is stored).

**Example**:

```rust
let err = Error::Config("invalid YAML".to_string());
assert_eq!(err.to_string(), "Configuration error: invalid YAML");
```

---

### `Metadata`

```rust
#[error("Failed to parse metadata in {path}")]
Metadata {
    path: PathBuf,
    #[source]
    source: Box<dyn CoreError + Send + Sync>,
},
```

- **Description**: Used when parsing metadata (e.g., front matter) fails. Stores the path of the problematic file and the original error.
- **Fields**:
  - `path: PathBuf` – The path to the source file where metadata parsing failed.
  - `source: Box<dyn CoreError + Send + Sync>` – The underlying error that caused the failure (boxed trait object).
- **Display**: `"Failed to parse metadata in {path}"`. The `{path}` placeholder prints the `PathBuf` using its `Display` implementation.
- **Source**: `source()` returns `Some(&*source)` (the boxed error as a `&dyn Error`), enabling error chain inspection.
- **Note**: The `#[source]` attribute tells `thiserror` to use this field as the error source. The `Box<dyn CoreError + Send + Sync>` allows storing any error type that is `Send + Sync`.

**Example**:

```rust
use std::path::PathBuf;

let source: Box<dyn core::error::Error + Send + Sync> =
    Box::new(std::io::Error::other("bad yaml"));
let err = Error::Metadata {
    path: PathBuf::from("content/post.md"),
    source,
};

assert_eq!(err.to_string(), "Failed to parse metadata in content/post.md");
let as_core_error: &dyn core::error::Error = &err;
let source_ref = as_core_error.source().unwrap();
assert_eq!(source_ref.to_string(), "bad yaml");
```

---

### `Render`

```rust
#[error("Template rendering error: {0}")]
Render(String),
```

- **Description**: Signifies an error during template rendering (e.g., missing variable, template not found, syntax error).
- **Fields**: A `String` describing the rendering problem.
- **Display**: `"Template rendering error: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::Render("template not found".to_string());
assert_eq!(err.to_string(), "Template rendering error: template not found");
```

---

### `Processor`

```rust
#[error("Content processor error: {0}")]
Processor(String),
```

- **Description**: Used when a content processor (e.g., Markdown parser, Sass compiler) fails.
- **Fields**: A `String` with details about the processor failure.
- **Display**: `"Content processor error: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::Processor("custom processor failed".to_string());
assert_eq!(err.to_string(), "Content processor error: custom processor failed");
```

---

### `Generator`

```rust
#[error("Generator error: {0}")]
Generator(String),
```

- **Description**: Represents an error in a generator component (e.g., RSS feed generation, sitemap creation).
- **Fields**: A `String` describing the generator error.
- **Display**: `"Generator error: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::Generator("RSS generation failed".to_string());
assert_eq!(err.to_string(), "Generator error: RSS generation failed");
```

---

### `PathTraversal`

```rust
#[error("Path traversal attempt detected: {0}")]
PathTraversal(String),
```

- **Description**: Indicates a path traversal attack was attempted or a path escapes a safe root directory.
- **Fields**: A `String` containing the offending path or description.
- **Display**: `"Path traversal attempt detected: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::PathTraversal("../escape".to_string());
assert_eq!(err.to_string(), "Path traversal attempt detected: ../escape");
```

---

### `MissingConfig`

```rust
#[error("Missing configuration key: {0}")]
MissingConfig(String),
```

- **Description**: Signals that a required configuration key is absent.
- **Fields**: A `String` naming the missing key.
- **Display**: `"Missing configuration key: {key}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::MissingConfig("base_url".to_string());
assert_eq!(err.to_string(), "Missing configuration key: base_url");
```

---

### `Generation`

```rust
#[error("Site generation error: {0}")]
Generation(String),
```

- **Description**: A general error during the site generation phase (e.g., failed to write output).
- **Fields**: A `String` with more information.
- **Display**: `"Site generation error: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::Generation("output write failed".to_string());
assert_eq!(err.to_string(), "Site generation error: output write failed");
```

---

### `NotFound`

```rust
#[error("Resource not found: {0}")]
NotFound(String),
```

- **Description**: Used when a requested resource (file, asset, page) cannot be found.
- **Fields**: A `String` identifying the missing resource.
- **Display**: `"Resource not found: {resource}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::NotFound("asset.css".to_string());
assert_eq!(err.to_string(), "Resource not found: asset.css");
```

---

### `Serialization`

```rust
#[error("Serialization error: {0}")]
Serialization(String),
```

- **Description**: Indicates a failure during serialization or deserialization (e.g., JSON conversion error).
- **Fields**: A `String` describing the serialization problem.
- **Display**: `"Serialization error: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::Serialization("invalid JSON".to_string());
assert_eq!(err.to_string(), "Serialization error: invalid JSON");
```

---

### `Validation`

```rust
#[error("Validation error: {0}")]
Validation(String),
```

- **Description**: Represents a validation failure (e.g., invalid input, constraint violation).
- **Fields**: A `String` explaining what failed validation.
- **Display**: `"Validation error: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::Validation("name too long".to_string());
assert_eq!(err.to_string(), "Validation error: name too long");
```

---

### `Duplicate`

```rust
#[error("Duplicate value: {0}")]
Duplicate(String),
```

- **Description**: Signals that a duplicate value was encountered where uniqueness was expected (e.g., duplicate key in a map).
- **Fields**: A `String` identifying the duplicated item.
- **Display**: `"Duplicate value: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::Duplicate("duplicate key".to_string());
assert_eq!(err.to_string(), "Duplicate value: duplicate key");
```

---

### `InvalidState`

```rust
#[error("Invalid state: {0}")]
InvalidState(String),
```

- **Description**: Indicates an unexpected program state (e.g., a null where a value is required, inconsistent internal data).
- **Fields**: A `String` describing the invalid state.
- **Display**: `"Invalid state: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::InvalidState("unexpected null".to_string());
assert_eq!(err.to_string(), "Invalid state: unexpected null");
```

---

### `Internal`

```rust
#[error("Internal error: {0}")]
Internal(String),
```

- **Description**: Used for internal errors that should not normally occur (e.g., bugs in the code, unreachable conditions).
- **Fields**: A `String` with details suitable for debugging.
- **Display**: `"Internal error: {message}"`.
- **Source**: `None`.

**Example**:

```rust
let err = Error::Internal("bug in code".to_string());
assert_eq!(err.to_string(), "Internal error: bug in code");
```

---

## `Result<T>` Type Alias

```rust
pub type Result<T> = core::result::Result<T, Error>;
```

- **Purpose**: A convenient alias so that functions can return `Result<T>` instead of the more verbose `std::result::Result<T, librawssg_error::Error>`.
- **Usage**: Throughout the `librawssg` ecosystem, functions that may fail with any of the above errors use this alias.

**Example**:

```rust
fn read_config(path: &str) -> librawssg_error::Result<String> {
    let content = std::fs::read_to_string(path)?; // `?` converts io::Error into Error::Io
    Ok(content)
}
```

---

## Error Sources and `std::error::Error`

All variants of `Error` implement `std::error::Error` (via `thiserror`). The `source()` method returns:

- For `Io`: `Some(&self.0)` (the underlying `io::Error`).
- For `Metadata`: `Some(self.source.as_ref())` (the boxed error).
- For all other variants: `None`.

This allows error chains to be inspected using `std::error::Error::source()`.

**Example** (from integration tests):

```rust
let source: Box<dyn core::error::Error + Send + Sync> =
    Box::new(std::io::Error::other("bad yaml"));
let err = Error::Metadata {
    path: PathBuf::from("content/post.md"),
    source,
};

let as_core_error: &dyn core::error::Error = &err;
let source_ref = as_core_error.source().unwrap();
assert_eq!(source_ref.to_string(), "bad yaml");
```

---

## Conversion from `std::io::Error`

The `Io` variant has the `#[from]` attribute, which automatically generates:

```rust
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}
```

This enables the `?` operator to convert `std::io::Error` into `Error` in any function returning `Result<T, Error>` (or `librawssg_error::Result<T>`).

**Example**:

```rust
fn read_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)?; // io::Error becomes Error::Io
    Ok(content)
}
```

---

## Usage Examples from Tests

The test suite provides excellent examples of how to construct and use the error type.

### Display Messages

Each variant has a test asserting its exact `Display` output. For example:

```rust
#[test]
fn display_for_config_error() {
    let err = Error::Config("invalid YAML".to_string());
    assert_eq!(err.to_string(), "Configuration error: invalid YAML");
}
```

All 15 variants have similar tests in `unit_tests.rs`.

### Using the `?` Operator

The integration test `io_error_propagates_via_question_mark` demonstrates how `?` works:

```rust
fn read_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

#[test]
fn io_error_propagates_via_question_mark() {
    let result = read_file("definitely_not_exists.txt");
    assert!(result.is_err());
    assert!(matches!(result, Err(Error::Io(_))));
}
```

### Source Chain

The `metadata_error_can_hold_boxed_dyn_error` test shows how to store an arbitrary error and retrieve it via `source()`:

```rust
let source: Box<dyn core::error::Error + Send + Sync> =
    Box::new(std::io::Error::other("bad yaml"));
let err = Error::Metadata {
    path: PathBuf::from("content/post.md"),
    source,
};

let as_core_error: &dyn core::error::Error = &err;
let source_ref = as_core_error.source();
assert!(source_ref.is_some());
```

### Property Tests

Property tests verify that the error message always preserves the input string exactly, regardless of content (including empty strings, newlines, special characters):

```rust
#[test]
fn config_error_message_preserves_input() {
    let samples = [
        "",
        "short",
        "a very long error message with symbols !@#$%^&*()",
        "line1\nline2",
    ];

    for sample in samples {
        let err = Error::Config(sample.to_string());
        assert_eq!(err.to_string(), format!("Configuration error: {sample}"));
    }
}
```

Similar tests exist for `PathTraversal` and `Render`.

---

## Guidelines for Error Usage

When writing code in the `librawssg` ecosystem, follow these recommendations:

- **Use the most specific variant** that describes the failure. For example:
  - I/O failures → `Error::Io`.
  - Missing file/resource → `Error::NotFound`.
  - Invalid user input → `Error::Validation`.
  - Security issue (path traversal) → `Error::PathTraversal`.
- **Attach context when possible** – Include the relevant path, key, or identifier in the error message string.
- **Preserve source errors** – If an underlying error is available, use the `Metadata` variant (or add a new variant with a `#[source]` field) to maintain the error chain.
- **Avoid matching exhaustively on `Error`** – Because the enum is non‑exhaustive, always include a catch‑all arm when matching to prevent future breakage.
- **Use `Result<T>` alias** for concise function signatures.

---

## Testing Suite Overview

The crate includes three test files:

- **`unit_tests.rs`** – Tests each variant’s `Display` message, `source()` for `Metadata`, `From<io::Error>` conversion, the `Result` alias, and `Debug` output.
- **`integration_tests.rs`** – Tests the `?` operator integration and the source chain for `Metadata` using a boxed dynamic error.
- **`property_tests.rs`** – Property‑based tests that verify error messages preserve arbitrary input strings for `Config`, `PathTraversal`, and `Render`.

Together, these tests ensure the error type is robust, easy to use, and consistent.

---

## Conclusion

`librawssg_error` provides a centralized, well‑structured error type for the entire static site generator project. With 15 descriptive variants, automatic `Display` and `Error` implementations, convenient conversion from `io::Error`, and support for error sources, it simplifies error handling across all modules. The non‑exhaustive design guarantees future extensibility without breaking downstream code.

For further details, refer to the source code and test files.
