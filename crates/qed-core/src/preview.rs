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
            let range = request
                .headers()
                .iter()
                .find(|header| header.field.equiv("Range"))
                .map(|header| header.value.as_str().to_string());
            serve_file(request, &file, range.as_deref())
        }
        Target::AddSlash(location) => {
            let header = Header::from_bytes("Location", location).expect("valid header");
            request.respond(Response::empty(301).with_header(header))
        }
        // The site's own 404 page, like a web server set up for it.
        Target::NotFound => match File::open(root.join("404.html")) {
            Ok(page) => request.respond(
                Response::from_file(page)
                    .with_status_code(404)
                    .with_header(content_type(Path::new("404.html"))),
            ),
            Err(_) => request.respond(Response::from_string("Not found").with_status_code(404)),
        },
    }
}

/// Sends a file, or the part of it a `Range: bytes=…` header asks for.
/// Browsers ask for parts of audio and video, to start playing before
/// the whole file is there and to jump to a later point; without them, an
/// episode's player can't seek.
fn serve_file(request: Request, file: &Path, range: Option<&str>) -> std::io::Result<()> {
    use std::io::{Read, Seek, SeekFrom};

    let mut opened = File::open(file)?;
    let size = opened.metadata()?.len();
    let accept = Header::from_bytes("Accept-Ranges", "bytes").expect("valid header");
    match range.map(|range| byte_range(range, size)) {
        None => request.respond(
            Response::from_file(opened)
                .with_header(content_type(file))
                .with_header(accept),
        ),
        Some(Some((start, end))) => {
            opened.seek(SeekFrom::Start(start))?;
            let length = end - start + 1;
            let content_range =
                Header::from_bytes("Content-Range", format!("bytes {start}-{end}/{size}"))
                    .expect("valid header");
            request.respond(Response::new(
                206.into(),
                vec![content_type(file), accept, content_range],
                opened.take(length),
                usize::try_from(length).ok(),
                None,
            ))
        }
        Some(None) => {
            let content_range = Header::from_bytes("Content-Range", format!("bytes */{size}"))
                .expect("valid header");
            request.respond(Response::empty(416).with_header(content_range))
        }
    }
}

/// Reads a `Range` header for a file of `size` bytes into the first and
/// last byte wanted: `bytes=100-199`, `bytes=100-` (to the end) and
/// `bytes=-100` (the last 100). `None` if it asks for nothing the file
/// has. Only the first range of a list is served.
fn byte_range(header: &str, size: u64) -> Option<(u64, u64)> {
    let spec = header
        .trim()
        .strip_prefix("bytes=")?
        .split(',')
        .next()?
        .trim();
    let (start, end) = spec.split_once('-')?;
    let last = size.checked_sub(1)?;
    let (start, end) = match (start.trim(), end.trim()) {
        ("", suffix) => {
            let count: u64 = suffix.parse().ok()?;
            (size.checked_sub(count.min(size))?, last)
        }
        (start, "") => (start.parse().ok()?, last),
        (start, end) => (start.parse().ok()?, end.parse::<u64>().ok()?.min(last)),
    };
    (start <= end && start <= last).then_some((start, end))
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
pub(crate) fn percent_decode(text: &str) -> String {
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
        "aac" => "audio/aac",
        "ogg" | "oga" | "opus" => "audio/ogg",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
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

    #[test]
    fn reads_byte_ranges() {
        assert_eq!(byte_range("bytes=0-99", 1000), Some((0, 99)));
        assert_eq!(byte_range("bytes=900-", 1000), Some((900, 999)));
        assert_eq!(byte_range("bytes=-100", 1000), Some((900, 999)));
        assert_eq!(byte_range("bytes=990-2000", 1000), Some((990, 999)));
        assert_eq!(byte_range("bytes=1000-", 1000), None);
        assert_eq!(byte_range("bytes=5-1", 1000), None);
        assert_eq!(byte_range("lines=1-2", 1000), None);
    }

    #[test]
    fn serves_parts_of_files_for_audio_players() {
        let dir = site();
        fs::write(dir.path().join("episode.mp3"), "0123456789").unwrap();
        let preview = Preview::start(dir.path()).unwrap();
        let url = format!("{}episode.mp3", preview.url());

        let part = get_with(&url, "Range: bytes=2-5\r\n");
        assert!(part.starts_with("HTTP/1.1 206"), "{part}");
        assert!(part.contains("Content-Range: bytes 2-5/10"), "{part}");
        assert!(part.contains("audio/mpeg"), "{part}");
        assert!(part.ends_with("\r\n\r\n2345"), "{part}");

        let whole = get(&url);
        assert!(whole.contains("Accept-Ranges: bytes"), "{whole}");
        assert!(whole.ends_with("0123456789"), "{whole}");

        let past = get_with(&url, "Range: bytes=20-\r\n");
        assert!(past.starts_with("HTTP/1.1 416"), "{past}");
    }

    /// A minimal HTTP GET, so the test needs no HTTP client library.
    fn get(url: &str) -> String {
        get_with(url, "")
    }

    /// A GET with more header lines, each ending in `\r\n`.
    fn get_with(url: &str, headers: &str) -> String {
        use std::io::{Read, Write};
        let rest = url.strip_prefix("http://").unwrap();
        let (host, path) = rest.split_at(rest.find('/').unwrap());
        let mut stream = std::net::TcpStream::connect(host).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {host}\r\n{headers}Connection: close\r\n\r\n"
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }
}
