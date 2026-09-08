# librawssg_fs

**Version**: 1.0.0 (implied)  
**Crate name**: `librawssg_fs`  
**Description**: A filesystem abstraction layer for static site generators. Defines the `FileSystem` trait with a comprehensive set of file and directory operations, and provides a concrete implementation `RealFs` that delegates to `std::fs` and `walkdir`. The trait includes built‑in path traversal protection and convenience methods for atomic operations.

---

## Table of Contents

1. [Overview](#overview)
2. [Modules](#modules)
3. [Trait `FileSystem`](#trait-filesystem)
   - [Trait Definition](#trait-definition)
   - [Required Methods](#required-methods)
     - [`read_to_string`](#read_to_string)
     - [`read_bytes`](#read_bytes)
     - [`write`](#write)
     - [`create_dir_all`](#create_dir_all)
     - [`remove_dir_all`](#remove_dir_all)
     - [`remove_file`](#remove_file)
     - [`create_dir`](#create_dir)
     - [`exists`](#exists)
     - [`is_dir`](#is_dir)
     - [`is_file`](#is_file)
     - [`read_dir`](#read_dir)
     - [`copy_file`](#copy_file)
     - [`copy_dir_all`](#copy_dir_all)
     - [`walk_dir`](#walk_dir)
     - [`canonicalize`](#canonicalize)
     - [`rename`](#rename)
     - [`atomic_write`](#atomic_write)
     - [`touch`](#touch)
     - [`metadata`](#metadata)
     - [`symlink_metadata`](#symlink_metadata)
     - [`permissions`](#permissions)
     - [`set_permissions`](#set_permissions)
     - [`read_link`](#read_link)
     - [`hard_link`](#hard_link)
   - [Provided (Default) Methods](#provided-default-methods)
     - [`is_symlink`](#is_symlink)
     - [`canonicalize_or_join`](#canonicalize_or_join)
     - [`safe_join`](#safe_join)
     - [`copy`](#copy)
     - [`rename_or_copy`](#rename_or_copy)
4. [Struct `RealFs`](#struct-realfs)
   - [Implementation Details](#implementation-details)
   - [Example Usage](#example-usage)
5. [Error Handling](#error-handling)
6. [Implementing a Custom `FileSystem`](#implementing-a-custom-filesystem)
7. [Security Considerations](#security-considerations)
8. [Testing Suite Overview](#testing-suite-overview)
9. [Complete Code Examples from Tests](#complete-code-examples-from-tests)

---

## Overview

`librawssg_fs` provides a trait‑based abstraction over filesystem operations. This allows static site generator components to interact with the filesystem without being tightly coupled to `std::fs`. It enables:

- **Testability**: Mock filesystems can be injected in unit tests.
- **Portability**: Different filesystem backends (e.g., in‑memory, virtual) can implement the trait.
- **Security**: Built‑in path traversal protection through `safe_join` and `canonicalize_or_join`.

The crate exports:

- `pub trait FileSystem` – The main abstraction.
- `pub struct RealFs` – A zero‑sized type that implements `FileSystem` using the real OS filesystem.

---

## Modules

The crate root (`lib.rs`) defines the `FileSystem` trait and re‑exports `RealFs` from the `real` module.

```rust
pub mod real;
pub use real::RealFs;
```

There is also an internal module `real.rs` containing the `RealFs` implementation.

---

## Trait `FileSystem`

The `FileSystem` trait is the core of this crate. It is object‑safe and requires implementors to be `Send + Sync` (safe to share across threads). The trait provides many required methods and several methods with default implementations.

```rust
pub trait FileSystem: Send + Sync {
    // Required methods (see below)
    // Provided methods with default implementations
}
```

### Required Methods

These methods **must** be implemented by any type that implements `FileSystem`. They map closely to `std::fs` functions and `walkdir` functionality.

#### `read_to_string`

```rust
fn read_to_string(&self, path: &Path) -> io::Result<String>;
```

**Purpose**: Reads the entire contents of a file into a `String`.

**Parameters**:

- `path`: The path to the file to read.

**Returns**: `Ok(String)` containing the file contents, or an `Err(io::Error)` if the file cannot be read (e.g., not found, permission denied, invalid UTF‑8).

**Example**:

```rust
let content = fs.read_to_string(Path::new("hello.txt"))?;
```

#### `read_bytes`

```rust
fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>>;
```

**Purpose**: Reads the entire contents of a file as raw bytes.

**Parameters**:

- `path`: The path to the file.

**Returns**: `Ok(Vec<u8>)` with the file bytes, or an `Err(io::Error)`.

**Example**:

```rust
let data = fs.read_bytes(Path::new("image.png"))?;
```

#### `write`

```rust
fn write(&self, path: &Path, content: &[u8]) -> io::Result<()>;
```

**Purpose**: Writes the given bytes to a file, creating any necessary parent directories.

**Parameters**:

- `path`: Destination file path.
- `content`: Bytes to write.

**Returns**: `Ok(())` on success, or `Err(io::Error)` on failure (e.g., permission denied, disk full).

**Behavior**: The default `RealFs` implementation creates parent directories before writing (via `create_dir_all` on the parent). This is convenient for writing deeply nested outputs.

**Example**:

```rust
fs.write(Path::new("a/b/c.txt"), b"hello")?;
```

#### `create_dir_all`

```rust
fn create_dir_all(&self, path: &Path) -> io::Result<()>;
```

**Purpose**: Creates a directory and all its missing parents.

**Parameters**:

- `path`: The directory path to create.

**Returns**: `Ok(())` or `Err(io::Error)`.

**Note**: Unlike `create_dir`, this does **not** error if the directory already exists.

**Example**:

```rust
fs.create_dir_all(Path::new("a/b/c"))?;
```

#### `remove_dir_all`

```rust
fn remove_dir_all(&self, path: &Path) -> io::Result<()>;
```

**Purpose**: Removes a directory and all its contents recursively.

**Parameters**:

- `path`: Directory path to remove.

**Returns**: `Ok(())` or `Err(io::Error)` (e.g., directory does not exist, permission denied).

**Warning**: This is destructive and cannot be undone.

#### `remove_file`

```rust
fn remove_file(&self, path: &Path) -> io::Result<()>;
```

**Purpose**: Deletes a single file.

**Parameters**:

- `path`: File path to remove.

**Returns**: `Ok(())` or `Err(io::Error)`.

#### `create_dir`

```rust
fn create_dir(&self, path: &Path) -> io::Result<()>;
```

**Purpose**: Creates a single directory. Fails if the parent directory does not exist or if the directory already exists.

**Parameters**:

- `path`: Directory path to create.

**Returns**: `Ok(())` or `Err(io::Error)` (e.g., already exists, parent missing).

#### `exists`

```rust
fn exists(&self, path: &Path) -> bool;
```

**Purpose**: Checks whether a path exists (as a file, directory, symlink, etc.).

**Parameters**:

- `path`: Path to check.

**Returns**: `true` if the path exists, `false` otherwise.

**Note**: This method does not follow symlinks for broken symlinks; it returns `false` for a broken symlink.

#### `is_dir`

```rust
fn is_dir(&self, path: &Path) -> bool;
```

**Purpose**: Checks whether the path points to a directory.

**Returns**: `true` if it is a directory, `false` otherwise (including if it does not exist).

#### `is_file`

```rust
fn is_file(&self, path: &Path) -> bool;
```

**Purpose**: Checks whether the path points to a regular file.

**Returns**: `true` if it is a regular file, `false` otherwise.

#### `read_dir`

```rust
fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
```

**Purpose**: Lists all entries (files and directories) directly inside a directory.

**Parameters**:

- `path`: Directory path.

**Returns**: `Ok(Vec<PathBuf>)` containing the full paths of all entries, or `Err(io::Error)`.

**Note**: The order is not guaranteed. It does not recurse into subdirectories.

#### `copy_file`

```rust
fn copy_file(&self, from: &Path, to: &Path) -> io::Result<u64>;
```

**Purpose**: Copies a file from `from` to `to`. If `to` already exists, it will be overwritten.

**Parameters**:

- `from`: Source file path.
- `to`: Destination file path.

**Returns**: `Ok(u64)` with the number of bytes copied, or `Err(io::Error)`.

**Note**: Does not create parent directories of `to` in the default `RealFs`; use `copy` or `copy_dir_all` for that.

#### `copy_dir_all`

```rust
fn copy_dir_all(&self, from: &Path, to: &Path) -> io::Result<()>;
```

**Purpose**: Recursively copies a directory tree from `from` to `to`. Creates the destination directory and all parent directories as needed.

**Parameters**:

- `from`: Source directory path.
- `to`: Destination directory path.

**Returns**: `Ok(())` or `Err(io::Error)`.

**Behavior**:

1. Creates `to` directory.
2. Walks all files in `from` (using `walk_dir`).
3. For each file, computes relative path and creates parent directories in `to`, then copies the file.

#### `walk_dir`

```rust
fn walk_dir(&self, root: &Path) -> io::Result<Vec<PathBuf>>;
```

**Purpose**: Recursively collects all **files** under `root`. Does not include directories or symlinks to directories.

**Parameters**:

- `root`: Root directory to traverse.

**Returns**: `Ok(Vec<PathBuf>)` with the full paths of all files, or `Err(io::Error)`.

**Note**: The default `RealFs` uses the `walkdir` crate to handle traversal. It follows symlinks? (The `WalkDir::new` default does not follow symlinks; it will include symlinks but not traverse into them unless `.follow_links(true)` is set. Here symlinks to files will be included? `entry.file_type().is_file()` will be true for a symlink to a file? Actually `file_type()` returns the type of the symlink itself, not the target, unless `follow_links` is used. So symlinks are not considered files and are skipped.)

#### `canonicalize`

```rust
fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
```

**Purpose**: Returns the canonical, absolute form of a path, resolving all symbolic links and normalizing `.` and `..` components.

**Parameters**:

- `path`: The path to canonicalize.

**Returns**: `Ok(PathBuf)` with the canonical path, or `Err(io::Error)` (e.g., path does not exist).

#### `rename`

```rust
fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
```

**Purpose**: Renames (moves) a file or directory from `from` to `to`. On most filesystems this is an atomic operation when source and destination are on the same filesystem.

**Parameters**:

- `from`: Source path.
- `to`: Destination path.

**Returns**: `Ok(())` or `Err(io::Error)`.

**Note**: If `to` exists, it may be overwritten (platform‑dependent). Does not work across different mount points (returns `CrossesDevices` error).

#### `atomic_write`

```rust
fn atomic_write(&self, path: &Path, content: &[u8]) -> io::Result<()>;
```

**Purpose**: Writes data to a file atomically by first writing to a temporary file and then renaming it over the target path.

**Parameters**:

- `path`: Destination file path.
- `content`: Bytes to write.

**Returns**: `Ok(())` or `Err(io::Error)`.

**Behavior**:

1. Creates a temporary file with extension `.tmp` (by calling `with_extension("tmp")` on the target path).
2. Writes the content to the temporary file (using `write`, which creates parent directories).
3. Renames the temporary file to the target path (using `rename`).
4. If the rename fails, attempts to remove the temporary file and returns the error.

**Note**: The temporary file name is derived from the target; it is not a hidden file and may collide if multiple writes happen concurrently to the same path. This is a best‑effort atomic write suitable for many use cases.

#### `touch`

```rust
fn touch(&self, path: &Path) -> io::Result<()>;
```

**Purpose**: Creates an empty file at `path` or updates its access/modification timestamp if it already exists.

**Parameters**:

- `path`: File path.

**Returns**: `Ok(())` or `Err(io::Error)`.

**Behavior**:

1. Creates parent directories (like `write`).
2. Opens the file in append/create mode, which creates it if missing.
3. Calls `sync_all()` to flush to disk (optional, but ensures metadata is updated).

**Note**: Existing file content is preserved.

#### `metadata`

```rust
fn metadata(&self, path: &Path) -> io::Result<std::fs::Metadata>;
```

**Purpose**: Returns metadata for a file or directory, following symlinks.

**Parameters**:

- `path`: Path to query.

**Returns**: `Ok(fs::Metadata)` or `Err(io::Error)`.

#### `symlink_metadata`

```rust
fn symlink_metadata(&self, path: &Path) -> io::Result<std::fs::Metadata>;
```

**Purpose**: Returns metadata for a path **without** following symlinks (i.e., metadata of the symlink itself).

**Parameters**:

- `path`: Path to query.

**Returns**: `Ok(fs::Metadata)` or `Err(io::Error)`.

#### `permissions`

```rust
fn permissions(&self, path: &Path) -> io::Result<std::fs::Permissions>;
```

**Purpose**: Reads the permissions of a file or directory.

**Parameters**:

- `path`: Path to query.

**Returns**: `Ok(fs::Permissions)` or `Err(io::Error)`.

**Note**: The default `RealFs` obtains permissions from `metadata`, which follows symlinks.

#### `set_permissions`

```rust
fn set_permissions(&self, path: &Path, permissions: std::fs::Permissions) -> io::Result<()>;
```

**Purpose**: Sets the permissions of a file or directory.

**Parameters**:

- `path`: Target path.
- `permissions`: New permissions.

**Returns**: `Ok(())` or `Err(io::Error)`.

#### `read_link`

```rust
fn read_link(&self, path: &Path) -> io::Result<PathBuf>;
```

**Purpose**: Reads the target of a symbolic link.

**Parameters**:

- `path`: Path to the symlink.

**Returns**: `Ok(PathBuf)` containing the link target, or `Err(io::Error)` if the path is not a symlink or does not exist.

#### `hard_link`

```rust
fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()>;
```

**Purpose**: Creates a hard link from `from` to `to`.

**Parameters**:

- `from`: Existing file path.
- `to`: New hard link path.

**Returns**: `Ok(())` or `Err(io::Error)`.

**Note**: Both paths must be on the same filesystem.

---

### Provided (Default) Methods

These methods have default implementations that rely on the required methods. Implementors may override them for performance or platform‑specific behavior.

#### `is_symlink`

```rust
fn is_symlink(&self, path: &Path) -> bool {
    self.symlink_metadata(path)
        .is_ok_and(|meta| meta.file_type().is_symlink())
}
```

**Purpose**: Checks whether the given path is a symbolic link.

**Returns**: `true` if the path is a symlink (even if broken), `false` otherwise.

**Implementation**: Uses `symlink_metadata` (which does not follow symlinks) and checks the file type.

**Example**:

```rust
if fs.is_symlink(Path::new("link")) { ... }
```

#### `canonicalize_or_join`

```rust
fn canonicalize_or_join(&self, base: &Path, candidate: &Path) -> io::Result<PathBuf>
```

**Purpose**: Safely resolves a possibly non‑existent path relative to `base`. If the joined path exists, it is canonicalized; otherwise it returns the canonical parent joined with the file name, after normalizing `.` and `..` components.

**Parameters**:

- `base`: The base directory (usually already canonical).
- `candidate`: A relative path (may contain `.` and `..`).

**Returns**: `Ok(PathBuf)` with the resolved path, or `Err(io::Error)` if path traversal is detected or other errors occur.

**Detailed Behavior**:

1. Normalizes the `candidate` path by iterating over its components:
   - `CurDir` (`.`) is ignored.
   - `ParentDir` (`..`) causes the last normal component to be popped. If there is no previous normal component (i.e., attempt to go above root), it returns `PermissionDenied` with message `"path traversal detected"`.
   - `Prefix` and `RootDir` components are pushed (though they are unusual for relative candidates and may cause issues later).
   - `Normal` components are pushed.
2. Joins the normalized candidate with `base`.
3. If the joined path exists, canonicalizes it (resolving symlinks, etc.).
4. If it does not exist:
   - Canonicalizes the parent directory of the joined path.
   - Appends the file name of the joined path to the canonical parent.
   - Returns that path.

**Security**: This method prevents `..` from escaping the base directory (unless there are symlinks that point outside; canonicalization of existing paths can still lead outside base, which is why `safe_join` adds an extra check). For non‑existent paths, the parent canonicalization ensures that the final path is within the canonical base.

**Example** (from tests):

```rust
let existing = base.join("existing.txt");
fs.write(&existing, b"data")?;

let canon_existing = fs.canonicalize_or_join(base, Path::new("existing.txt"))?;
let canon_direct = fs.canonicalize(&existing)?;
assert_eq!(canon_existing, canon_direct);

let missing = Path::new("missing.txt");
let canon_missing = fs.canonicalize_or_join(base, missing)?;
let canon_base = fs.canonicalize(base)?;
assert_eq!(canon_missing, canon_base.join(missing));
```

#### `safe_join`

```rust
fn safe_join(&self, base: &Path, candidate: &Path) -> io::Result<PathBuf>
```

**Purpose**: Safely joins a candidate path to a base directory, ensuring the result is **within** the base directory (no path traversal). This is the recommended way to compute destination paths for user‑provided or untrusted relative paths.

**Parameters**:

- `base`: The base directory (can be relative; it will be canonicalized internally).
- `candidate`: A relative path (may contain `.` and `..`).

**Returns**: `Ok(PathBuf)` with the resolved path, guaranteed to start with the canonical base. Returns `Err(io::Error)` with `PermissionDenied` if the resolved path escapes the base (e.g., `candidate = "../secret"`).

**Implementation Details**:

1. Canonicalizes `base`.
2. Calls `canonicalize_or_join` with the canonical base and `candidate`.
3. Checks that the resulting path starts with the canonical base. If not, returns `PermissionDenied`.

**Why needed**: Although `canonicalize_or_join` prevents simple `..` traversal, symlinks inside the base directory could cause a resolved path to point outside the base even after normalization. The `starts_with` check enforces containment.

**Example**:

```rust
let base = tmp.path().join("base");
fs.create_dir_all(&base)?;

let safe = fs.safe_join(&base, Path::new("inside.txt"))?;
assert!(safe.starts_with(&base));

let traversal = Path::new("../escape.txt");
let result = fs.safe_join(&base, traversal);
assert!(result.is_err());
```

#### `copy`

```rust
fn copy(&self, from: &Path, to: &Path) -> io::Result<()>
```

**Purpose**: Copies a file or directory from `from` to `to`. If `from` is a directory, it recursively copies the whole tree; if it is a file, it performs a single file copy.

**Parameters**:

- `from`: Source path.
- `to`: Destination path.

**Returns**: `Ok(())` or `Err(io::Error)`.

**Implementation**:

```rust
if self.is_dir(from) {
    self.copy_dir_all(from, to)
} else {
    self.copy_file(from, to).map(|_| ())
}
```

**Note**: Does not create parent directories of `to` for file copies (unless `copy_file` implementation does; the default `RealFs::copy_file` does not). For directories, `copy_dir_all` does create `to` and parents as needed.

**Example**:

```rust
fs.copy(&src_file, &dst_file)?;
fs.copy(&src_dir, &dst_dir)?;
```

#### `rename_or_copy`

```rust
fn rename_or_copy(&self, from: &Path, to: &Path) -> io::Result<()>
```

**Purpose**: Attempts to rename `from` to `to`. If the rename fails with `ErrorKind::CrossesDevices` (i.e., source and destination are on different filesystems), it falls back to copying the directory tree and then removing the source.

**Parameters**:

- `from`: Source path.
- `to`: Destination path.

**Returns**: `Ok(())` or `Err(io::Error)`.

**Behavior**:

1. Try `rename(from, to)`.
2. If success, return `Ok(())`.
3. If error kind is `CrossesDevices`:
   - `copy_dir_all(from, to)` to copy contents.
   - `remove_dir_all(from)` to delete source.
   - Return `Ok(())`.
4. Otherwise, return the original error.

**Note**: The fallback only works for directories (as the code uses `copy_dir_all` and `remove_dir_all`). For a file across devices, this will likely fail. This method is useful for moving directories across mount points.

**Example**:

```rust
fs.rename_or_copy(&src_dir, &dst_dir)?;
```

---

## Struct `RealFs`

`RealFs` is a zero‑sized struct that implements `FileSystem` by delegating directly to the operating system’s filesystem APIs.

```rust
#[derive(Debug, Default, Clone, Copy)]
pub struct RealFs;
```

It has no fields and can be instantiated with `RealFs` or `RealFs::default()`.

### Implementation Details

`RealFs` uses:

- `std::fs` for most operations.
- `walkdir::WalkDir` for `walk_dir`.
- The `tracing::instrument` attribute is applied to most methods for logging (though `tracing` is not enabled by default; it can be used with a subscriber).

All methods follow the behavior described in the trait definitions. The `write` method creates parent directories before writing, and `atomic_write` uses a temporary `.tmp` file.

### Example Usage

```rust
use librawssg_fs::{FileSystem, RealFs};
use std::path::Path;

let fs = RealFs;

// Write a file
fs.write(Path::new("output/file.txt"), b"Hello")?;

// Read it back
let content = fs.read_to_string(Path::new("output/file.txt"))?;
assert_eq!(content, "Hello");

// Create directory
fs.create_dir_all(Path::new("output/sub"))?;

// Copy directory
fs.copy_dir_all(Path::new("output"), Path::new("backup"))?;
```

---

## Error Handling

All methods that can fail return `io::Result<T>` (i.e., `Result<T, std::io::Error>`). This is the standard error type from the standard library, so no custom error enum is defined in this crate. Consumers can inspect the error kind (e.g., `ErrorKind::NotFound`, `PermissionDenied`, `CrossesDevices`) to handle specific failures.

The provided security methods (`canonicalize_or_join` and `safe_join`) return `io::Error` with `ErrorKind::PermissionDenied` when path traversal is detected, along with the message `"path traversal detected"`.

---

## Implementing a Custom `FileSystem`

To create a mock filesystem or an alternative backend, implement the `FileSystem` trait. You must provide implementations for all 24 required methods. The provided methods can be left as default unless you need custom behavior.

**Example of a minimal mock** (from tests, adapted):

```rust
use librawssg_fs::FileSystem;
use std::io;
use std::path::{Path, PathBuf};

struct DummyFs;

impl FileSystem for DummyFs {
    fn read_to_string(&self, _path: &Path) -> io::Result<String> {
        Err(io::Error::other("not implemented"))
    }
    // ... implement all other required methods similarly
    // (returning Err or trivial values)
}
```

Because `FileSystem` is `Send + Sync`, your mock must also be thread‑safe. In practice, you can use `Arc` or interior mutability if state is needed.

---

## Security Considerations

The library includes two methods specifically designed to prevent path traversal attacks:

- **`canonicalize_or_join`**: Normalizes `..` and `.` and ensures the final path does not go above the base (in terms of lexical components). However, it may still follow symlinks that point outside the base if the path exists.
- **`safe_join`**: Combines `canonicalize_or_join` with a `starts_with` check on the canonical base, providing a stronger guarantee that the result is contained within the base directory.

**Recommendation**: Always use `safe_join` when constructing output paths from untrusted input (e.g., user‑supplied relative URLs). Avoid using `join` directly followed by canonicalization without containment checks.

---

## Testing Suite Overview

The test file `tests/filesystem.rs` contains comprehensive tests for `RealFs` and the provided methods. It uses `tempfile::TempDir` to create isolated temporary directories. The tests cover:

- Basic read/write operations (string and bytes)
- Creating directories and files
- Error cases for non‑existent paths
- `copy_file`, `copy_dir_all`, `rename`, `remove_file`, `remove_dir_all`
- `atomic_write` (including nested paths and overwriting)
- `touch` (creating new and preserving existing content)
- `walk_dir` (recursive collection)
- `canonicalize_or_join` (existing and missing paths)
- `safe_join` (rejecting traversal, allowing dot segments inside)
- Metadata and permissions
- Symlink and hard link operations (Unix only)

All tests can be run with `cargo test`.

---

## Complete Code Examples from Tests

Below are selected examples from the test suite that illustrate common usage patterns. They can be copied and adapted.

### Writing and Reading a String

```rust
use librawssg_fs::{FileSystem, RealFs};
use std::path::Path;
use tempfile::TempDir;

let tmp = TempDir::new().unwrap();
let fs = RealFs;
let file_path = tmp.path().join("hello.txt");

fs.write(&file_path, b"world").unwrap();
let content = fs.read_to_string(&file_path).unwrap();
assert_eq!(content, "world");
```

### Atomic Write Overwriting

```rust
let file = tmp.path().join("atomic.txt");
fs.atomic_write(&file, b"first").unwrap();
fs.atomic_write(&file, b"second").unwrap();
let content = fs.read_to_string(&file).unwrap();
assert_eq!(content, "second");
```

### Safe Join Blocking Traversal

```rust
let base = tmp.path().join("base");
fs.create_dir_all(&base).unwrap();

let safe = fs.safe_join(&base, Path::new("inside.txt")).unwrap();
assert!(safe.starts_with(&base));

let traversal = Path::new("../escape.txt");
assert!(fs.safe_join(&base, traversal).is_err());
```

### Copying a Directory Recursively

```rust
let src = tmp.path().join("src_dir");
let dst = tmp.path().join("dst_dir");
fs.create_dir_all(&src.join("nested")).unwrap();
fs.write(&src.join("file1.txt"), b"one").unwrap();
fs.write(&src.join("nested").join("file2.txt"), b"two").unwrap();

fs.copy_dir_all(&src, &dst).unwrap();
assert!(fs.exists(&dst.join("file1.txt")));
assert!(fs.exists(&dst.join("nested").join("file2.txt")));
```

### Using `walk_dir` to Gather All Files

```rust
let root = tmp.path().join("root");
fs.create_dir_all(&root.join("sub")).unwrap();
fs.write(&root.join("root.txt"), b"root").unwrap();
fs.write(&root.join("sub").join("sub.txt"), b"sub").unwrap();

let files = fs.walk_dir(&root).unwrap();
assert_eq!(files.len(), 2);
```

---

## Summary

`librawssg_fs` provides a robust, thread‑safe filesystem abstraction with built‑in path traversal protection and convenience methods for atomic operations and cross‑device moves. The `RealFs` implementation is ready to use, and the trait enables easy mocking for unit tests. The extensive test suite validates all features and serves as living documentation.

For any additional details, refer to the source code and inline comments.
