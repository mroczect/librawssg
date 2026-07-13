use librawssg::fs::{FileSystem, real::RealFs};
use tempfile::tempdir;

#[test]
fn real_fs_read_write_cycle() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    RealFs.write(&file_path, b"hello").unwrap();
    assert!(RealFs.exists(&file_path));
    let content = RealFs.read_to_string(&file_path).unwrap();
    assert_eq!(content, "hello");
}

#[test]
fn real_fs_create_and_remove_dir() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    RealFs.create_dir_all(&sub).unwrap();
    assert!(RealFs.is_dir(&sub));
    RealFs.remove_dir_all(&sub).unwrap();
    assert!(!RealFs.exists(&sub));
}

#[test]
fn real_fs_walk_dir() {
    let dir = tempdir().unwrap();
    RealFs.write(&dir.path().join("a.txt"), b"a").unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();
    RealFs.write(&dir.path().join("sub/b.txt"), b"b").unwrap();
    let files = RealFs.walk_dir(dir.path()).unwrap();
    assert!(files.contains(&dir.path().join("a.txt")));
    assert!(files.contains(&dir.path().join("sub/b.txt")));
}

#[test]
fn real_fs_copy_file() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    RealFs.write(&src, b"copy").unwrap();
    RealFs.copy_file(&src, &dst).unwrap();
    assert_eq!(RealFs.read_to_string(&dst).unwrap(), "copy");
}
