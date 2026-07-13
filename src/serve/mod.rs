#[cfg(feature = "serve")]
mod server {
    use crate::error::RawssgError;
    use std::path::{Path, PathBuf};
    use std::thread;
    use tiny_http::{Header, Response, Server};

    fn mime_type(path: &Path) -> String {
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
        .to_string()
    }

    fn handle_request(request: tiny_http::Request, dist_path: &Path) -> Result<(), RawssgError> {
        let url = request.url().to_string();
        let requested_path = if url == "/" {
            "index.html".to_string()
        } else {
            url.trim_start_matches('/').to_string()
        };

        let file_path = dist_path.join(&requested_path);
        match file_path.canonicalize() {
            Ok(canon) if canon.starts_with(dist_path) => match std::fs::read(&canon) {
                Ok(content) => {
                    let mime = mime_type(&canon);
                    let header = Header::from_bytes("Content-Type", mime.as_str())
                        .expect("invalid MIME header");
                    let response = Response::from_data(content).with_header(header);
                    request.respond(response).ok();
                }
                Err(_) => {
                    let response =
                        Response::from_string("500 Internal Server Error").with_status_code(500);
                    request.respond(response).ok();
                }
            },
            _ => {
                let response = Response::from_string("404 Not Found").with_status_code(404);
                request.respond(response).ok();
            }
        }
        Ok(())
    }

    pub fn start_dev_server(output_dir: &Path, port: u16) -> Result<(), RawssgError> {
        let dist = output_dir
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
