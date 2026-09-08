use librawssg_fs::{FileSystem, RealFs};
use std::path::Path;
use tempfile::TempDir;
use tracing as _;
use walkdir as _;

macro_rules! must {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                eprintln!("{} failed: {}", $context, err);
                std::process::exit(1);
            }
        }
    };
}

#[test]
fn write_and_read_string() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file_path = tmp.path().join("hello.txt");

    must!(fs.write(&file_path, b"world"), "write");
    let content = must!(fs.read_to_string(&file_path), "read_to_string");
    assert_eq!(content, "world");
}

#[test]
fn write_and_read_bytes() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file_path = tmp.path().join("data.bin");
    let data = vec![0, 1, 2, 3];

    must!(fs.write(&file_path, &data), "write");
    let read = must!(fs.read_bytes(&file_path), "read_bytes");
    assert_eq!(read, data);
}

#[test]
fn read_nonexistent_file_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let result = fs.read_to_string(&tmp.path().join("missing.txt"));
    assert!(result.is_err());
}

#[test]
fn write_creates_parent_directories() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let nested = tmp.path().join("a/b/c/file.txt");

    must!(fs.write(&nested, b"deep"), "write nested");
    assert!(fs.exists(&nested));
    let content = must!(fs.read_to_string(&nested), "read nested");
    assert_eq!(content, "deep");
}

#[test]
fn create_dir_and_check_exists() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let dir_path = tmp.path().join("subdir");

    must!(fs.create_dir(&dir_path), "create_dir");
    assert!(fs.exists(&dir_path));
    assert!(fs.is_dir(&dir_path));
    assert!(!fs.is_file(&dir_path));
}

#[test]
fn create_dir_already_exists_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let dir = tmp.path().join("existing_dir");
    must!(fs.create_dir(&dir), "create_dir first");

    let result = fs.create_dir(&dir);
    assert!(result.is_err(), "creating existing dir should error");
}

#[test]
fn create_dir_all_recursive() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let nested = tmp.path().join("a/b/c");

    must!(fs.create_dir_all(&nested), "create_dir_all");
    assert!(fs.exists(&nested));
    assert!(fs.is_dir(&nested));
}

#[test]
fn read_dir_lists_entries() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let dir = tmp.path().join("dir");
    must!(fs.create_dir_all(&dir), "create_dir_all");
    must!(fs.write(&dir.join("a.txt"), b"a"), "write a");
    must!(fs.write(&dir.join("b.txt"), b"b"), "write b");

    let entries = must!(fs.read_dir(&dir), "read_dir");
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|p| p.ends_with("a.txt")));
    assert!(entries.iter().any(|p| p.ends_with("b.txt")));
}

#[test]
fn read_dir_empty_returns_empty_vec() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let dir = tmp.path().join("empty");
    must!(fs.create_dir(&dir), "create_dir");

    let entries = must!(fs.read_dir(&dir), "read_dir");
    assert!(entries.is_empty());
}

#[test]
fn remove_dir_all_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let dir = tmp.path().join("dir");
    must!(fs.create_dir_all(&dir.join("sub")), "create_dir_all");
    must!(fs.write(&dir.join("file.txt"), b"data"), "write");

    must!(fs.remove_dir_all(&dir), "remove_dir_all");
    assert!(!fs.exists(&dir));
}

#[test]
fn remove_dir_all_nonexistent_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let result = fs.remove_dir_all(&tmp.path().join("ghost"));
    assert!(result.is_err());
}

#[test]
fn copy_file_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    must!(fs.write(&src, b"data"), "write");

    let copied = must!(fs.copy_file(&src, &dst), "copy_file");
    assert!(copied > 0);
    let content = must!(fs.read_to_string(&dst), "read dst");
    assert_eq!(content, "data");
}

#[test]
fn copy_file_nonexistent_source_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let result = fs.copy_file(&tmp.path().join("missing"), &tmp.path().join("dest"));
    assert!(result.is_err());
}

#[test]
fn rename_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let old = tmp.path().join("old.txt");
    let new = tmp.path().join("new.txt");
    must!(fs.write(&old, b"content"), "write");

    must!(fs.rename(&old, &new), "rename");
    assert!(!fs.exists(&old));
    assert!(fs.exists(&new));
    let content = must!(fs.read_to_string(&new), "read new");
    assert_eq!(content, "content");
}

#[test]
fn rename_nonexistent_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let result = fs.rename(&tmp.path().join("missing"), &tmp.path().join("dest"));
    assert!(result.is_err());
}

#[test]
fn remove_file_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("delete.txt");
    must!(fs.write(&file, b"to delete"), "write");

    must!(fs.remove_file(&file), "remove_file");
    assert!(!fs.exists(&file));
}

#[test]
fn remove_file_nonexistent_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let result = fs.remove_file(&tmp.path().join("missing.txt"));
    assert!(result.is_err());
}

#[test]
fn atomic_write_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("atomic.txt");

    must!(fs.atomic_write(&file, b"first"), "atomic_write first");
    let content = must!(fs.read_to_string(&file), "read first");
    assert_eq!(content, "first");

    must!(fs.atomic_write(&file, b"second"), "atomic_write second");
    let content = must!(fs.read_to_string(&file), "read second");
    assert_eq!(content, "second");
}

#[test]
fn atomic_write_creates_parents() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("parent/child/atomic.txt");

    must!(fs.atomic_write(&file, b"nested"), "atomic_write nested");
    assert!(fs.exists(&file));
    let content = must!(fs.read_to_string(&file), "read nested");
    assert_eq!(content, "nested");
}

#[test]
fn touch_creates_file() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("new_file.txt");

    must!(fs.touch(&file), "touch");
    assert!(fs.exists(&file));
    let content = must!(fs.read_to_string(&file), "read touched");
    assert_eq!(content, "");
}

#[test]
fn touch_existing_file_keeps_content() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("existing.txt");
    must!(fs.write(&file, b"keep"), "write");

    must!(fs.touch(&file), "touch");
    let content = must!(fs.read_to_string(&file), "read after touch");
    assert_eq!(content, "keep");
}

#[test]
fn copy_dir_all_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let src_dir = tmp.path().join("src_dir");
    let dst_dir = tmp.path().join("dst_dir");
    must!(fs.create_dir_all(&src_dir.join("nested")), "create_dir_all");
    must!(fs.write(&src_dir.join("file1.txt"), b"one"), "write file1");
    must!(
        fs.write(&src_dir.join("nested").join("file2.txt"), b"two"),
        "write file2"
    );

    must!(fs.copy_dir_all(&src_dir, &dst_dir), "copy_dir_all");
    assert!(fs.exists(&dst_dir.join("file1.txt")));
    assert!(fs.exists(&dst_dir.join("nested").join("file2.txt")));
    let content = must!(fs.read_to_string(&dst_dir.join("file1.txt")), "read file1");
    assert_eq!(content, "one");
}

#[test]
fn copy_file_via_copy_method() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    must!(fs.write(&src, b"copy me"), "write src");

    must!(fs.copy(&src, &dst), "copy file");
    assert!(fs.exists(&dst));
    let content = must!(fs.read_to_string(&dst), "read dst");
    assert_eq!(content, "copy me");
}

#[test]
fn copy_dir_via_copy_method() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let src = tmp.path().join("src_dir");
    let dst = tmp.path().join("dst_dir");
    must!(fs.create_dir_all(&src), "create_dir_all");
    must!(fs.write(&src.join("file.txt"), b"dir copy"), "write file");

    must!(fs.copy(&src, &dst), "copy dir");
    assert!(fs.exists(&dst.join("file.txt")));
    let content = must!(fs.read_to_string(&dst.join("file.txt")), "read dst file");
    assert_eq!(content, "dir copy");
}

#[test]
fn walk_dir_collects_files_recursively() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let root = tmp.path().join("root");
    must!(fs.create_dir_all(&root.join("sub")), "create_dir_all");
    must!(fs.write(&root.join("root.txt"), b"root"), "write root");
    must!(
        fs.write(&root.join("sub").join("sub.txt"), b"sub"),
        "write sub"
    );

    let files = must!(fs.walk_dir(&root), "walk_dir");
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|p| p.ends_with("root.txt")));
    assert!(files.iter().any(|p| p.ends_with("sub.txt")));
}

#[test]
fn walk_dir_empty_directory_returns_empty() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let dir = tmp.path().join("empty");
    must!(fs.create_dir(&dir), "create_dir");

    let files = must!(fs.walk_dir(&dir), "walk_dir");
    assert!(files.is_empty());
}

#[test]
fn canonicalize_or_join_works_for_existing_and_missing() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let base = tmp.path();
    let existing = base.join("existing.txt");
    must!(fs.write(&existing, b"data"), "write");

    let canon_existing = must!(
        fs.canonicalize_or_join(base, Path::new("existing.txt")),
        "canonicalize existing"
    );
    let canon_direct = must!(fs.canonicalize(&existing), "canonicalize direct");
    assert_eq!(canon_existing, canon_direct);

    let missing = Path::new("missing.txt");
    let canon_missing = must!(
        fs.canonicalize_or_join(base, missing),
        "canonicalize missing"
    );
    let canon_base = must!(fs.canonicalize(base), "canonicalize base");
    assert_eq!(canon_missing, canon_base.join(missing));
}

#[test]
fn safe_join_blocks_traversal() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let base = tmp.path().join("base");
    must!(fs.create_dir_all(&base), "create base");

    let safe = must!(
        fs.safe_join(&base, Path::new("inside.txt")),
        "safe join inside"
    );
    assert!(safe.starts_with(&base));

    let traversal = Path::new("../escape.txt");
    let result = fs.safe_join(&base, traversal);
    assert!(result.is_err(), "traversal should be rejected");
}

#[test]
fn safe_join_allows_dot_segments_inside() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let base = tmp.path().join("base");
    must!(fs.create_dir_all(&base), "create base");

    let candidate = Path::new("sub/../file.txt");
    let result = must!(
        fs.safe_join(&base, candidate),
        "safe join with dot segments"
    );
    let expected = must!(fs.canonicalize(&base), "canonicalize base").join("file.txt");
    assert_eq!(result, expected);
}

#[test]
fn metadata_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("meta.txt");
    must!(fs.write(&file, b"hello"), "write");

    let meta = must!(fs.metadata(&file), "metadata");
    assert_eq!(meta.len(), 5);
    assert!(meta.is_file());
}

#[test]
fn metadata_nonexistent_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let result = fs.metadata(&tmp.path().join("missing.txt"));
    assert!(result.is_err());
}

#[test]
fn symlink_metadata_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("file.txt");
    must!(fs.write(&file, b"data"), "write");

    let meta = must!(fs.symlink_metadata(&file), "symlink_metadata");
    assert!(meta.is_file());
}

#[test]
fn permissions_roundtrip() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("perm.txt");
    must!(fs.write(&file, b"data"), "write");

    let mut perms = must!(fs.permissions(&file), "permissions");
    perms.set_mode(0o600);
    must!(fs.set_permissions(&file, perms.clone()), "set_permissions");

    let read_perms = must!(fs.permissions(&file), "permissions after set");
    assert_eq!(read_perms.mode() & 0o777, 0o600);
}

#[test]
fn permissions_nonexistent_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let result = fs.permissions(&tmp.path().join("missing.txt"));
    assert!(result.is_err());
}
#[test]
fn set_permissions_nonexistent_errors() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let perms = std::fs::Permissions::from_mode(0o644);
    let result = fs.set_permissions(&tmp.path().join("missing.txt"), perms);
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn symlink_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let target = tmp.path().join("target.txt");
    let link = tmp.path().join("link.txt");
    must!(fs.write(&target, b"target"), "write target");

    must!(std::os::unix::fs::symlink(&target, &link), "symlink");
    assert!(fs.is_symlink(&link));
    let link_target = must!(fs.read_link(&link), "read_link");
    assert_eq!(link_target, target);
    let content = must!(fs.read_to_string(&link), "read through symlink");
    assert_eq!(content, "target");
}

#[cfg(unix)]
#[test]
fn is_symlink_false_for_regular_file() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let file = tmp.path().join("regular.txt");
    must!(fs.write(&file, b"data"), "write");
    assert!(!fs.is_symlink(&file));
}

#[cfg(unix)]
#[test]
fn hard_link_works() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let original = tmp.path().join("original.txt");
    let hard = tmp.path().join("hard.txt");
    must!(fs.write(&original, b"shared"), "write original");

    must!(fs.hard_link(&original, &hard), "hard_link");
    assert!(fs.exists(&hard));
    let content = must!(fs.read_to_string(&hard), "read hard");
    assert_eq!(content, "shared");
}

#[cfg(unix)]
#[test]
fn hard_link_nonexistent_source_errors() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let result = fs.hard_link(
        &tmp.path().join("missing.txt"),
        &tmp.path().join("hard.txt"),
    );
    assert!(result.is_err());
}

#[test]
fn rename_or_copy_fallback_renames_on_same_device() {
    let tmp = must!(TempDir::new(), "TempDir::new");
    let fs = RealFs;
    let src = tmp.path().join("src_dir");
    let dst = tmp.path().join("dst_dir");
    must!(fs.create_dir_all(&src), "create src");
    must!(fs.write(&src.join("file.txt"), b"data"), "write file");

    must!(fs.rename_or_copy(&src, &dst), "rename_or_copy");
    assert!(!fs.exists(&src));
    assert!(fs.exists(&dst));
    let content = must!(fs.read_to_string(&dst.join("file.txt")), "read dst");
    assert_eq!(content, "data");
}
