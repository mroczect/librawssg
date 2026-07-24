#[cfg(feature = "serve")]
mod server {
    use crate::error::RawssgError;
    use crate::fs::real::RealFs;
    use crate::util::safe_path;
    use std::path::{Path, PathBuf};
    use std::thread;
    use tiny_http::{Header, Response, Server};

    fn mime_type(path: &Path) -> &'static str {
        match path.extension().and_then(|e| e.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("xml") => "application/xml",
            Some("svg") => "image/svg+xml",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            _ => "application/octet-stream",
        }
    }

    fn handle_request(request: tiny_http::Request, dist_path: &Path) -> Result<(), RawssgError> {
        let url = request.url().to_string();
        let requested_path = if url == "/" {
            "index.html"
        } else {
            url.trim_start_matches('/')
        };

        let candidate = Path::new(requested_path);

        match safe_path(&RealFs, dist_path, candidate) {
            Ok(safe_path) => match std::fs::read(&safe_path) {
                Ok(content) => {
                    let mime = mime_type(&safe_path);
                    let header = Header::from_bytes("Content-Type", mime.as_bytes())
                        .unwrap_or_else(|_| {
                            Header::from_bytes("Content-Type", b"application/octet-stream").unwrap()
                        });
                    let response = Response::from_data(content).with_header(header);
                    request.respond(response).ok();
                }
                Err(_) => {
                    let response =
                        Response::from_string("500 Internal Server Error").with_status_code(500);
                    request.respond(response).ok();
                }
            },
            Err(_) => {
                let response = Response::from_string("404 Not Found").with_status_code(404);
                request.respond(response).ok();
            }
        }
        Ok(())
    }

    pub fn start_dev_server(output_dir: &Path, port: u16) -> Result<(), RawssgError> {
        let dist: PathBuf = output_dir
            .canonicalize()
            .unwrap_or_else(|_| output_dir.to_path_buf());

        let server = Server::http(format!("0.0.0.0:{}", port))
            .map_err(|e| RawssgError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        tracing::info!("dev server listening on http://localhost:{}", port);
        for request in server.incoming_requests() {
            let dist = dist.clone();
            thread::spawn(move || {
                if let Err(e) = handle_request(request, &dist) {
                    tracing::error!("request error: {:?}", e);
                }
            });
        }
        Ok(())
    }
}

#[cfg(feature = "serve")]
pub use server::start_dev_server;
