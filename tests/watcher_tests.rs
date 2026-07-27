// tests/watcher_tests.rs
#![cfg(feature = "serve")]
use librawssg::serve::watch_dirs;
use std::sync::mpsc;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn watcher_triggers_on_file_create() {
    let dir = tempdir().unwrap();
    let (tx, rx) = mpsc::channel();
    let on_change = move || {
        tx.send(()).ok();
    };
    let _watcher = watch_dirs(&[dir.path().to_path_buf()], on_change).expect("watcher start");

    std::fs::write(dir.path().join("new.md"), b"content").unwrap();

    let received = rx.recv_timeout(Duration::from_secs(3));
    assert!(received.is_ok(), "watcher should have triggered");
}
