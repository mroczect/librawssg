#![cfg(feature = "serve")]
use librawssg::fs::{FileSystem, real::RealFs};
use librawssg::serve::start_dev_server;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

fn http_get(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("cannot connect");
    let request = format!("GET {} HTTP/1.0\r\nHost: localhost\r\n\r\n", path);
    stream.write_all(request.as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn find_available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind")
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn server_serves_index_html() {
    let dir = tempdir().unwrap();
    let index_path = dir.path().join("index.html");
    RealFs.write(&index_path, b"<h1>Hello</h1>").unwrap();

    let port = find_available_port();
    let dist = dir.path().to_path_buf();
    let server_handle = thread::spawn(move || {
        let _ = start_dev_server(&dist, port);
    });

    thread::sleep(Duration::from_millis(500));

    let response = http_get(port, "/");
    assert!(response.contains("200 OK") || response.contains("HTTP/1.0 200"));
    assert!(response.contains("<h1>Hello</h1>"));

    drop(server_handle);
}

#[test]
fn server_returns_500_for_missing_file() {
    let dir = tempdir().unwrap();
    let port = find_available_port();
    let dist = dir.path().to_path_buf();
    let server_handle = thread::spawn(move || {
        let _ = start_dev_server(&dist, port);
    });

    let mut response = String::new();
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        if let Ok(res) = std::panic::catch_unwind(|| http_get(port, "/nope.html")) {
            response = res;
            break;
        }
    }

    assert!(
        response.contains("500"),
        "Expected 500 status, got: {}",
        response
    );

    drop(server_handle);
}
