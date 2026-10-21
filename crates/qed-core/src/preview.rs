//! A small web server for looking at a built site before publishing it.
//!
//! It listens on `127.0.0.1` only, on a free port the system picks, and
//! serves files from one folder. It stops when the [`Preview`] is dropped.

use std::fs::File;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use tiny_http::{Header, Request, Response, Server};

pub struct Preview {
    root: PathBuf,
    url: String,
    server: Arc<Server>,
    thread: Option<JoinHandle<()>>,
}

impl Preview {
    /// Starts serving `root` in a background thread.
    pub fn start(root: &Path) -> std::io::Result<Preview> {
        let server = Server::http("127.0.0.1:0").map_err(std::io::Error::other)?;
        let address = server
            .server_addr()
            .to_ip()
            .ok_or_else(|| std::io::Error::other("preview server has no IP address"))?;
        let server = Arc::new(server);

        // The thread gets its own handle to the server and its own copy of
        // the root path; `move` hands both over to it.
        let thread = {
            let server = Arc::clone(&server);
            let root = root.to_path_buf();
            thread::spawn(move || {
                // `recv` fails once `unblock` is called in `drop`.
                while let Ok(request) = server.recv() {
                    let _ = respond(&root, request);
                }
            })
        };

        Ok(Preview {
            root: root.to_path_buf(),
            url: format!("http://{address}/"),
            server,
            thread: Some(thread),
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for Preview {
    fn drop(&mut self) {
        self.server.unblock();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// What a request path maps to.
#[derive(Debug, PartialEq)]
enum Target {
    File(PathBuf),
    /// A folder requested without its trailing slash: `/about` → `/about/`.
    AddSlash(String),
    NotFound,
}

fn respond(root: &Path, request: Request) -> std::io::Result<()> {
    let path = request
        .url()
        .split(['?', '#'])
        .next()
        .unwrap_or("/")
        .to_string();
    match resolve(root, &path) {
        Target::File(file) => {
            let response = Response::from_file(File::open(&file)?).with_header(content_type(&file));
            request.respond(response)
        }
        Target::AddSlash(location) => {
            let header = Header::from_bytes("Location", location).expect("valid header");
            request.respond(Response::empty(301).with_header(header))
        }
        Target::NotFound => {
            request.respond(Response::from_string("Not found").with_status_code(404))
        }
    }
}

fn resolve(root: &Path, url_path: &str) -> Target {
    let decoded = percent_decode(url_path);
    let relative = Path::new(decoded.trim_start_matches('/'));

    // Only plain names: no `..`, no absolute paths.
    if !relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        return Target::NotFound;
    }

    let full = root.join(relative);
    if full.is_dir() {
        if url_path.ends_with('/') {
            let index = full.join("index.html");
            return if index.is_file() {
                Target::File(index)
            } else {
                Target::NotFound
            };
        }
        return Target::AddSlash(format!("{url_path}/"));
    }
    if full.is_file() {
        Target::File(full)
    } else {
        Target::NotFound
    }
}

/// Decodes `%20` and friends. Invalid sequences are kept as they are.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = text.get(i + 1..i + 3)
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn content_type(file: &Path) -> Header {
    let mime = match file.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "xml" => "application/xml",
        "json" => "application/json",
        "txt" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    };
    Header::from_bytes("Content-Type", mime).expect("valid header")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn site() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("about")).unwrap();
        fs::write(dir.path().join("index.html"), "home").unwrap();
        fs::write(dir.path().join("about/index.html"), "about").unwrap();
        fs::write(dir.path().join("my file.txt"), "spaces").unwrap();
        dir
    }

    #[test]
    fn maps_paths_to_files() {
        let dir = site();
        let root = dir.path();
        assert_eq!(resolve(root, "/"), Target::File(root.join("index.html")));
        assert_eq!(
            resolve(root, "/about/"),
            Target::File(root.join("about/index.html"))
        );
        assert_eq!(resolve(root, "/about"), Target::AddSlash("/about/".into()));
        assert_eq!(
            resolve(root, "/my%20file.txt"),
            Target::File(root.join("my file.txt"))
        );
        assert_eq!(resolve(root, "/missing"), Target::NotFound);
    }

    #[test]
    fn refuses_to_leave_the_folder() {
        let dir = site();
        assert_eq!(resolve(dir.path(), "/../etc/passwd"), Target::NotFound);
        assert_eq!(resolve(dir.path(), "/%2e%2e/etc/passwd"), Target::NotFound);
    }

    #[test]
    fn serves_files_over_http() {
        let dir = site();
        let preview = Preview::start(dir.path()).unwrap();
        assert!(preview.url().starts_with("http://127.0.0.1:"));

        let body = get(&format!("{}about/", preview.url()));
        assert!(body.starts_with("HTTP/1.1 200"), "{body}");
        assert!(body.contains("text/html"), "{body}");
        assert!(body.ends_with("about"), "{body}");
    }

    /// A minimal HTTP GET, so the test needs no HTTP client library.
    fn get(url: &str) -> String {
        use std::io::{Read, Write};
        let rest = url.strip_prefix("http://").unwrap();
        let (host, path) = rest.split_at(rest.find('/').unwrap());
        let mut stream = std::net::TcpStream::connect(host).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }
}
