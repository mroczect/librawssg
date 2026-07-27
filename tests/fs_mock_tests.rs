mod common;
use common::MockFs;
use librawssg::fs::FileSystem;
use std::path::Path;

#[test]
fn read_to_string_existing_file() {
    let mut fs = MockFs::new();
    fs.add_file("/test.txt", "hello");
    let content = fs.read_to_string(Path::new("/test.txt")).unwrap();
    assert_eq!(content, "hello");
}

#[test]
fn read_to_string_not_found() {
    let fs = MockFs::new();
    assert!(fs.read_to_string(Path::new("/missing.txt")).is_err());
}

#[test]
fn read_bytes_existing() {
    let mut fs = MockFs::new();
    fs.add_file("/data.bin", "\x00\x01");
    let bytes = fs.read_bytes(Path::new("/data.bin")).unwrap();
    assert_eq!(bytes, vec![0, 1]);
}

#[test]
fn exists_true() {
    let mut fs = MockFs::new();
    fs.add_file("/file.txt", "x");
    assert!(fs.exists(Path::new("/file.txt")));
}

#[test]
fn exists_false() {
    let fs = MockFs::new();
    assert!(!fs.exists(Path::new("/ghost.txt")));
}

#[test]
fn is_dir_and_is_file() {
    let mut fs = MockFs::new();
    fs.add_file("/dir/file.txt", "data");
    assert!(fs.is_dir(Path::new("/dir")));
    assert!(fs.is_file(Path::new("/dir/file.txt")));
    assert!(!fs.is_dir(Path::new("/dir/file.txt")));
}

#[test]
fn read_dir_returns_entries() {
    let mut fs = MockFs::new();
    fs.add_file("/root/a.md", "a");
    fs.add_file("/root/b.md", "b");
    fs.dirs.push(Path::new("/root/sub").to_path_buf());
    let entries = fs.read_dir(Path::new("/root")).unwrap();
    assert_eq!(entries.len(), 3);
}

#[test]
fn walk_dir_recursive() {
    let mut fs = MockFs::new();
    fs.add_file("/root/a.md", "a");
    fs.add_file("/root/sub/b.md", "b");
    let files = fs.walk_dir(Path::new("/root")).unwrap();
    assert!(files.contains(&Path::new("/root/a.md").to_path_buf()));
    assert!(files.contains(&Path::new("/root/sub/b.md").to_path_buf()));
}

#[test]
fn read_to_string_mock_error() {
    let mut fs = MockFs::new();
    fs.add_file("/faulty.txt", "data");
    fs.read_error = Some(Path::new("/faulty.txt").to_path_buf());
    assert!(fs.read_to_string(Path::new("/faulty.txt")).is_err());
}
