# librawssg_handler

**Version**: 1.0.0 (implied)  
**Crate name**: `librawssg_handler`  
**Description**: Core data structures and traits for building a static site generator (SSG) handler. Provides `Document`, `Metadata`, and a `Processor` trait for processing content files.

---

## Table of Contents

1. [Overview](#overview)
2. [Modules](#modules)
3. [Struct `Document`](#struct-document)
   - [Fields](#document-fields)
   - [Constructor `new()`](#document-new)
   - [Method `relative_url()`](#document-relative_url)
   - [Method `add_taxonomy()`](#document-add_taxonomy)
   - [Method `depth()`](#document-depth)
   - [Method `with_list_items()`](#document-with_list_items)
4. [Struct `Metadata`](#struct-metadata)
   - [Fields](#metadata-fields)
   - [Constructor `new()`](#metadata-new)
   - [Method `is_draft()`](#metadata-is_draft)
   - [Method `insert_extra()`](#metadata-insert_extra)
   - [Method `get_extra()`](#metadata-get_extra)
   - [Serialization & Deserialization](#metadata-serialization)
   - [Default Implementation](#metadata-default)
5. [Trait `Processor`](#trait-processor)
   - [Required Methods](#processor-required-methods)
   - [Provided Methods](#processor-provided-methods)
   - [Implementing the Trait](#processor-implementation)
6. [Error Handling](#error-handling)
7. [External Traits & Types](#external-traits-and-types)
8. [Examples from Tests](#examples-from-tests)
9. [Validation Rules Summary](#validation-rules-summary)
10. [Testing Suite Overview](#testing-suite-overview)

---

## Overview

`librawssg_handler` is the core library for a static site generator. It defines the essential data structures used to represent a processed document (`Document`) and its front matter (`Metadata`). Additionally, it provides a pluggable `Processor` trait that allows different file types to be processed into `Document` instances.

The crate is intended to be used in conjunction with:

- `librawssg_error`: Provides the `Error` and `Result` types for consistent error handling.
- `librawssg_fs`: Defines a `FileSystem` trait abstracting file I/O operations (used by `Processor`).

---

## Modules

The library is organized into three public modules:

- **`document`** – Contains the `Document` struct.
- **`metadata`** – Contains the `Metadata` struct.
- **`processor`** – Contains the `Processor` trait.

All public types are re‑exported at the crate root for convenience:

```rust
pub use document::Document;
pub use metadata::Metadata;
pub use processor::Processor;
```

---

## Struct `Document`

Represents a fully processed content item ready for rendering or further processing.

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Document {
    pub metadata: Metadata,
    pub body: String,
    pub url: String,
    pub output_path: PathBuf,
    pub source_path: PathBuf,
    pub depth: usize,
    pub content_type: String,
    pub is_list: bool,
    pub list_items: Option<Vec<Self>>,
    pub taxonomies: HashMap<String, Vec<String>>,
}
```

### Document Fields

| Field          | Type                           | Description                                                                                        |
| -------------- | ------------------------------ | -------------------------------------------------------------------------------------------------- |
| `metadata`     | `Metadata`                     | Front matter metadata associated with the document.                                                |
| `body`         | `String`                       | The processed content body (e.g., rendered HTML, Markdown text, etc.).                             |
| `url`          | `String`                       | The relative URL where the document will be accessible (e.g., `"blog/my-post.html"`).              |
| `output_path`  | `PathBuf`                      | Filesystem path where the final output file should be written (e.g., `"blog/my-post/index.html"`). |
| `source_path`  | `PathBuf`                      | Path to the original source file (e.g., `"content/blog/my-post.md"`).                              |
| `depth`        | `usize`                        | Depth of the document in the site hierarchy (0 for top‑level). Used for sorting or navigation.     |
| `content_type` | `String`                       | Identifier for the kind of content (e.g., `"blog"`, `"page"`, `"article"`).                        |
| `is_list`      | `bool`                         | Indicates whether this document represents a list of other documents (e.g., an index page).        |
| `list_items`   | `Option<Vec<Document>>`        | If `is_list` is true, may contain the child documents. `None` otherwise or when not set.           |
| `taxonomies`   | `HashMap<String, Vec<String>>` | A map of taxonomy names (e.g., `"categories"`, `"tags"`) to lists of terms.                        |

> **Note:** The `#[non_exhaustive]` attribute means that external crates cannot exhaustively match on `Document` or construct it with a struct literal; they must use the provided constructor or update syntax. This allows adding fields in the future without breaking downstream code.

### Document::new

```rust
pub fn new(
    metadata: Metadata,
    body: impl Into<String>,
    url: impl Into<String>,
    output_path: impl Into<PathBuf>,
    source_path: impl Into<PathBuf>,
    depth: usize,
    content_type: impl Into<String>,
    is_list: bool,
) -> Result<Self>
```

**Purpose**: Creates a new `Document` after validating the provided arguments.

**Parameters**:

- `metadata`: A fully constructed `Metadata` instance.
- `body`: The content body (accepts any type convertible to `String`).
- `url`: The desired relative URL (must not be empty or whitespace only).
- `output_path`: The target output path (must not be empty).
- `source_path`: The source file path (must have a file name component).
- `depth`: The hierarchy depth (must be ≤ 1000).
- `content_type`: A string identifying the content type (e.g., `"blog"`, `"page"`).
- `is_list`: Boolean indicating whether this document is a list container.

**Returns**:

- `Ok(Document)` on success.
- `Err(librawssg_error::Error::Validation(message))` if any validation rule fails.

**Validation Rules**:

1. `url` must not be empty or contain only whitespace.
2. `output_path` must not be empty (as an OS string).
3. `source_path` must have a file name (i.e., its last component is not `..` or empty).
4. `depth` must not exceed 1000.

**Example**:

```rust
use librawssg_handler::{Document, Metadata};
use std::path::PathBuf;

let metadata = Metadata::new("My Post", "A short description")?;

let document = Document::new(
    metadata,
    "<h1>Hello</h1><p>World</p>",
    "blog/my-post.html",
    "blog/my-post/index.html",
    "content/blog/my-post.md",
    1,
    "blog",
    false,
)?;
```

---

### Document::relative_url

```rust
#[must_use]
pub fn relative_url(&self) -> &str
```

**Purpose**: Returns the relative URL of the document.

**Returns**: A string slice referencing the `url` field.

**Example**:

```rust
let doc = /* ... */;
assert_eq!(doc.relative_url(), "blog/my-post.html");
```

---

### Document::add_taxonomy

```rust
pub fn add_taxonomy(&mut self, name: impl Into<String>, items: Vec<String>)
```

**Purpose**: Inserts or replaces a taxonomy entry in the document’s `taxonomies` map.

**Parameters**:

- `name`: Taxonomy name (converted into `String`).
- `items`: A vector of string terms belonging to that taxonomy.

**Behavior**: If a taxonomy with the same name already exists, its value is replaced.

**Example**:

```rust
let mut doc = /* ... */;
doc.add_taxonomy("categories", vec!["rust".to_string(), "ssg".to_string()]);
assert_eq!(doc.taxonomies["categories"], vec!["rust", "ssg"]);
```

---

### Document::depth

```rust
#[must_use]
pub const fn depth(&self) -> usize
```

**Purpose**: Returns the `depth` field.

**Returns**: The document’s depth as a `usize`.

**Example**:

```rust
let doc = /* ... */;
assert_eq!(doc.depth(), 1);
```

---

### Document::with_list_items

```rust
#[must_use]
pub fn with_list_items(mut self, items: Vec<Self>) -> Self
```

**Purpose**: Consumes the document, sets its `list_items` field to `Some(items)`, and returns the modified document.

**Parameters**:

- `items`: A vector of `Document` instances that are children of this list document.

**Returns**: The same document with `list_items` set.

**Example**:

```rust
let parent = Document::new(/* ... */)?;
let child1 = Document::new(/* ... */)?;
let child2 = Document::new(/* ... */)?;
let list_doc = parent.with_list_items(vec![child1, child2]);
assert!(list_doc.list_items.is_some());
```

---

## Struct `Metadata`

Represents front matter (metadata) for a document. The struct is serializable and deserializable, making it suitable for parsing from formats like YAML or TOML front matter.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct Metadata {
    pub title: String,
    pub description: String,
    pub author: Option<String>,
    pub repo_url: Option<String>,
    pub license: Option<String>,
    pub date: Option<NaiveDate>,
    pub updated: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub draft: bool,
    pub extra: HashMap<String, serde_json::Value>,
}
```

### Metadata Fields

| Field         | Type                                 | Description                                                                  |
| ------------- | ------------------------------------ | ---------------------------------------------------------------------------- |
| `title`       | `String`                             | The document title (required, cannot be empty).                              |
| `description` | `String`                             | A short description of the content.                                          |
| `author`      | `Option<String>`                     | The author’s name, if known.                                                 |
| `repo_url`    | `Option<String>`                     | URL to the source repository.                                                |
| `license`     | `Option<String>`                     | License identifier (e.g., `"MIT"`, `"Apache-2.0"`).                          |
| `date`        | `Option<NaiveDate>`                  | Publication date (ISO 8601 date, e.g., `2026-09-08`).                        |
| `updated`     | `Option<NaiveDate>`                  | Last modification date.                                                      |
| `tags`        | `Vec<String>`                        | List of tags (keywords) associated with the content.                         |
| `draft`       | `bool`                               | If true, the document is considered a draft and may be excluded from builds. |
| `extra`       | `HashMap<String, serde_json::Value>` | Arbitrary extra key–value pairs for custom metadata.                         |

> **Note:** `#[non_exhaustive]` prevents exhaustive struct literals outside the crate; use the provided constructors or update syntax.

### Metadata::new

```rust
pub fn new(
    title: impl Into<String>,
    description: impl Into<String>,
) -> librawssg_error::Result<Self>
```

**Purpose**: Creates a `Metadata` instance with a required `title` and `description`. All other fields are set to their default values.

**Parameters**:

- `title`: The title (must not be empty or whitespace only).
- `description`: A description string.

**Returns**:

- `Ok(Metadata)` on success.
- `Err(librawssg_error::Error::Validation("metadata title cannot be empty"))` if the title is empty or whitespace.

**Example**:

```rust
use librawssg_handler::Metadata;

let meta = Metadata::new("My Title", "My Description")?;
assert_eq!(meta.title, "My Title");
assert!(!meta.draft);
assert!(meta.tags.is_empty());
```

---

### Metadata::is_draft

```rust
#[must_use]
pub const fn is_draft(&self) -> bool
```

**Purpose**: Returns the `draft` field.

**Returns**: `true` if the document is marked as a draft, otherwise `false`.

**Example**:

```rust
let mut meta = Metadata::new("Title", "Desc")?;
assert!(!meta.is_draft());
meta.draft = true;
assert!(meta.is_draft());
```

---

### Metadata::insert_extra

```rust
pub fn insert_extra(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>)
```

**Purpose**: Inserts or updates an entry in the `extra` map.

**Parameters**:

- `key`: The key (converted to `String`).
- `value`: Any type convertible to `serde_json::Value` (e.g., strings, numbers, booleans, arrays, objects, or `serde_json::json!` macro results).

**Behavior**: If the key already exists, its value is overwritten.

**Example**:

```rust
use serde_json::json;

let mut meta = Metadata::new("Title", "Desc")?;
meta.insert_extra("key", "value");
meta.insert_extra("number", 42);
meta.insert_extra("flag", true);
meta.insert_extra("nested", json!({"foo": "bar"}));
```

---

### Metadata::get_extra

```rust
#[must_use]
pub fn get_extra(&self, key: &str) -> Option<&serde_json::Value>
```

**Purpose**: Retrieves a reference to the value stored under `key` in the `extra` map.

**Parameters**:

- `key`: The key to look up.

**Returns**:

- `Some(&Value)` if the key exists.
- `None` otherwise.

**Example**:

```rust
let meta = /* ... */;
if let Some(v) = meta.get_extra("key") {
    assert_eq!(v, &json!("value"));
}
```

---

### Metadata Serialization

`Metadata` derives both `Serialize` and `Deserialize`, so it can be converted to/from JSON, YAML, etc. This is particularly useful for reading front matter from source files.

**Serialization Example**:

```rust
use serde_json;

let meta = Metadata::new("Hello", "World")?;
let json_str = serde_json::to_string(&meta)?;
// {"title":"Hello","description":"World","author":null,...}
```

**Deserialization Example**:

```rust
let json_str = r#"{
    "title": "Hello",
    "description": "World",
    "author": "Alice",
    "date": "2026-09-08",
    "tags": ["rust", "ssg"],
    "draft": false,
    "extra": {"foo": "bar"}
}"#;

let meta: Metadata = serde_json::from_str(json_str)?;
assert_eq!(meta.author.as_deref(), Some("Alice"));
```

---

### Metadata Default

The `Default` trait is implemented. All fields are set to sensible empty values:

- `title`: empty string
- `description`: empty string
- `author`, `repo_url`, `license`, `date`, `updated`: `None`
- `tags`: empty vector
- `draft`: `false`
- `extra`: empty `HashMap`

**Example**:

```rust
let meta = Metadata::default();
assert_eq!(meta.title, "");
assert!(!meta.draft);
assert!(meta.tags.is_empty());
```

---

## Trait `Processor`

The `Processor` trait defines an interface for components that can transform a source file into a `Document`. Multiple processors may be registered and invoked based on their ability to handle a given file.

```rust
pub trait Processor: Send + Sync {
    fn name(&self) -> &str;

    fn priority(&self) -> i32 {
        0
    }

    fn can_process(&self, relative_path: &Path, original_path: &Path) -> bool;

    fn process(
        &self,
        fs: &dyn FileSystem,
        relative_path: &Path,
        content_dir: &Path,
    ) -> Result<Option<Document>>;
}
```

### Processor Required Methods

#### `name()`

```rust
fn name(&self) -> &str
```

**Purpose**: Returns a human‑readable identifier for the processor (e.g., `"markdown"`, `"sass"`).

#### `can_process()`

```rust
fn can_process(&self, relative_path: &Path, original_path: &Path) -> bool
```

**Purpose**: Determines whether this processor should handle the given file.

**Parameters**:

- `relative_path`: The path of the file relative to the content directory.
- `original_path`: The full original path (often the same as `content_dir.join(relative_path)`).

**Returns**: `true` if the processor can process this file; `false` otherwise.

**Typical Implementation**: Check file extension or other attributes.

**Example**:

```rust
fn can_process(&self, relative_path: &Path, _original_path: &Path) -> bool {
    relative_path.extension().and_then(|e| e.to_str()) == Some("md")
}
```

#### `process()`

```rust
fn process(
    &self,
    fs: &dyn FileSystem,
    relative_path: &Path,
    content_dir: &Path,
) -> Result<Option<Document>>
```

**Purpose**: Reads the source file, processes it, and returns an optional `Document`.

**Parameters**:

- `fs`: A reference to a `FileSystem` implementation for performing I/O operations.
- `relative_path`: Path of the source file relative to the content directory.
- `content_dir`: The root directory containing all source content.

**Returns**:

- `Ok(Some(document))` if processing succeeded and produced a document.
- `Ok(None)` if the processor decides not to produce a document (e.g., the file is ignored).
- `Err(librawssg_error::Error)` if an error occurred during processing.

**Note**: The `FileSystem` trait is defined in the `librawssg_fs` crate. It abstracts many common file operations, allowing processors to be tested with mock filesystems.

---

### Processor Provided Methods

#### `priority()`

```rust
fn priority(&self) -> i32 {
    0
}
```

**Purpose**: Returns the priority of this processor. Processors with higher priority are invoked before those with lower priority. The default is `0`.

**Usage**: Allows ordering of processors when multiple might handle the same file.

**Example**:

```rust
fn priority(&self) -> i32 {
    10
}
```

---

### Processor Implementation

To create a custom processor, implement the `Processor` trait. Below is a complete example based on the test suite:

```rust
use librawssg_error::Result;
use librawssg_fs::FileSystem;
use librawssg_handler::{Document, Metadata, Processor};
use std::path::Path;

struct MarkdownProcessor;

impl Processor for MarkdownProcessor {
    fn name(&self) -> &str {
        "markdown"
    }

    fn can_process(&self, relative_path: &Path, _original_path: &Path) -> bool {
        relative_path.extension().and_then(|e| e.to_str()) == Some("md")
    }

    fn process(
        &self,
        fs: &dyn FileSystem,
        relative_path: &Path,
        content_dir: &Path,
    ) -> Result<Option<Document>> {
        // Read the source file
        let source_path = content_dir.join(relative_path);
        let content = fs.read_to_string(&source_path)?;

        // Parse front matter and body (simplified here)
        let metadata = Metadata::new("Untitled", "")?;
        let body = content; // In reality, you would render Markdown to HTML

        // Construct Document
        let doc = Document::new(
            metadata,
            body,
            relative_path.with_extension("html").to_string_lossy().to_string(),
            relative_path.with_extension("index.html").to_string_lossy().into(),
            source_path,
            1,
            "page",
            false,
        )?;

        Ok(Some(doc))
    }
}
```

---

## Error Handling

The library uses the `librawssg_error::Error` enum for all fallible operations. Relevant variants:

- `Error::Validation(String)` – Used when a validation rule fails (e.g., empty URL, invalid depth).
- `Error::Processor(String)` – Used by processors to signal processing errors.
- Other variants may exist but are not directly used in this crate.

The return type `Result<T>` is an alias for `std::result::Result<T, librawssg_error::Error>`.

**Example of Validation Error**:

```rust
let result = Document::new(meta, "body", "", "out", "src.md", 0, "page", false);
assert!(matches!(
    result,
    Err(librawssg_error::Error::Validation(ref msg)) if msg == "document url cannot be empty"
));
```

---

## External Traits & Types

### `FileSystem` Trait

The `Processor::process` method takes a `&dyn FileSystem`. This trait is defined in `librawssg_fs` and provides a comprehensive set of file operations (read, write, create directories, walk, etc.). A typical implementation wraps `std::fs`, but for testing, mock implementations are often used.

A minimal `FileSystem` implementation (used in tests) might implement all methods returning `io::Error::other("not implemented")` for those not needed.

---

## Examples from Tests

The test suite contains numerous examples that demonstrate correct usage and error conditions. Below are selected examples.

### Creating a Valid Document

```rust
use librawssg_handler::{Document, Metadata};

let metadata = Metadata::new("Title", "Description")?;
let doc = Document::new(
    metadata,
    "<p>Body</p>",
    "blog/my-post.html",
    "blog/my-post/index.html",
    "content/blog/my-post.md",
    1,
    "blog",
    false,
)?;

assert_eq!(doc.metadata.title, "Title");
assert_eq!(doc.body, "<p>Body</p>");
```

### Handling Invalid URL

```rust
let result = Document::new(
    Metadata::new("Title", "Desc")?,
    "body",
    "", // empty URL
    "out",
    "src.md",
    0,
    "page",
    false,
);
assert!(result.is_err());
```

### Adding Taxonomy Terms

```rust
let mut doc = /* ... */;
doc.add_taxonomy("categories", vec!["rust".to_string(), "ssg".to_string()]);
```

### Working with `extra` Metadata

```rust
use serde_json::json;

let mut meta = Metadata::new("Title", "Desc")?;
meta.insert_extra("key", "value");
if let Some(v) = meta.get_extra("key") {
    assert_eq!(v, &json!("value"));
}
```

### Processor Mock Example

```rust
struct MockProcessor { /* ... */ }

impl Processor for MockProcessor {
    fn name(&self) -> &str { "mock" }
    fn can_process(&self, _: &Path, _: &Path) -> bool { true }
    fn process(&self, _fs: &dyn FileSystem, _rel: &Path, _cd: &Path) -> Result<Option<Document>> {
        Ok(Some(document))
    }
}
```

---

## Validation Rules Summary

### `Metadata::new`

- `title` must not be empty or contain only whitespace.

### `Document::new`

1. `url` must not be empty or contain only whitespace.
2. `output_path` must not be empty (as an OS string).
3. `source_path` must have a file name component.
4. `depth` must be ≤ 1000.

Any violation results in an `Err(Error::Validation(...))`.

---

## Testing Suite Overview

The tests are organized into four files:

1. **`document_tests.rs`** – Validates `Document` construction, field defaults, error cases, and methods (`relative_url`, `add_taxonomy`, `depth`, `with_list_items`).
2. **`metadata_tests.rs`** – Tests `Metadata` creation, validation, `is_draft`, `insert_extra`/`get_extra`, default values, and JSON serialization/deserialization round‑trip.
3. **`processor_tests.rs`** – Tests the `Processor` trait using mock implementations: `name`, `priority`, `can_process`, `process` returning `Some`, `None`, and error.
4. **`unit_tests.rs`** – Verifies that all public items are re‑exported at the crate root.

All tests can serve as executable examples of the API usage.

---

## Conclusion

This documentation covers the public API of `librawssg_handler` in detail. The crate provides a flexible foundation for building static site generators by separating metadata handling (`Metadata`), document representation (`Document`), and pluggable processing logic (`Processor`). The validation rules ensure data integrity, and the use of traits like `FileSystem` enables testability.

For further details, refer to the source code and the accompanying test suite.
