use std::{env, fs::File, io, path::Path};
use tiny_http::{Header, Method, Request, Response, Server};

const WEB_ROOT: &str = "target/web";
const ASSETS: &[(&str, &str)] = &[
    ("index.html", "text/html; charset=utf-8"),
    ("dario.wasm", "application/wasm"),
    ("game.js", "text/javascript; charset=utf-8"),
    ("game.css", "text/css; charset=utf-8"),
    ("favicon.svg", "image/svg+xml"),
    ("vendor/mq_js_bundle.js", "text/javascript; charset=utf-8"),
    ("vendor/LICENSE-MIT", "text/plain; charset=utf-8"),
    ("vendor/LICENSE-APACHE", "text/plain; charset=utf-8"),
    ("vendor/NOTICE.txt", "text/plain; charset=utf-8"),
];

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root = Path::new(WEB_ROOT);
    for (name, _) in ASSETS {
        let path = root.join(name);
        let file = File::open(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("{}: {error}. Build or copy target/web first, then run from the project folder.", path.display()),
            )
        })?;
        if !file.metadata()?.is_file() {
            return Err(format!("{} is not a file", path.display()).into());
        }
    }

    let address = env::var("DARIO_ADDR").unwrap_or_else(|_| "127.0.0.1:3041".into());
    let server = Server::http(&address)?;
    println!(
        "Dario listening on http://{} (serving {})",
        server.server_addr(),
        root.canonicalize()?.display()
    );
    loop {
        let request = server.recv()?;
        if let Err(error) = respond(request, root) {
            eprintln!("Could not send response: {error}");
        }
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("valid static HTTP header")
}

fn respond(request: Request, root: &Path) -> io::Result<()> {
    if !matches!(request.method(), Method::Get | Method::Head) {
        return request.respond(
            Response::from_string("Method not allowed\n")
                .with_status_code(405)
                .with_header(header("Allow", "GET, HEAD")),
        );
    }

    let path = request.url().split('?').next().unwrap_or("/");
    let name = if path == "/" {
        "index.html"
    } else {
        path.strip_prefix('/').unwrap_or("")
    };
    // Only these public assets are routed; URLs never become filesystem paths.
    let Some(&(asset, content_type)) = ASSETS.iter().find(|(asset, _)| *asset == name) else {
        return request.respond(Response::from_string("Not found\n").with_status_code(404));
    };
    let file = match File::open(root.join(asset)) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Could not read {asset}: {error}");
            return request
                .respond(Response::from_string("Game asset unavailable\n").with_status_code(503));
        }
    };
    // tiny_http suppresses the body for HEAD while retaining the file's length.
    request.respond(
        Response::from_file(file)
            .with_header(header("Content-Type", content_type))
            .with_header(header("Cache-Control", "no-cache"))
            .with_header(header("X-Content-Type-Options", "nosniff")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::{Read, Write},
        net::TcpStream,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
        thread,
        time::Duration,
    };

    struct WebFiles(PathBuf);

    impl WebFiles {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = env::temp_dir().join(format!(
                "dario-server-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            fs::write(path.join("index.html"), b"<title>Dario</title>").unwrap();
            fs::write(path.join("dario.wasm"), b"\0asm\x01\0\0\0").unwrap();
            fs::write(path.join("private.txt"), b"must not be served").unwrap();
            Self(path)
        }

        fn request(&self, method: &str, path: &str) -> (String, Vec<u8>) {
            let server = Server::http("127.0.0.1:0").unwrap();
            let address = server.server_addr().to_ip().unwrap();
            let root = self.0.clone();
            let worker = thread::spawn(move || {
                let request = server
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap()
                    .expect("request within timeout");
                respond(request, &root).unwrap();
            });
            let mut stream = TcpStream::connect(address).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            write!(
                stream,
                "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
            )
            .unwrap();
            let mut bytes = Vec::new();
            stream.read_to_end(&mut bytes).unwrap();
            worker.join().unwrap();
            let split = bytes
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .unwrap();
            (
                String::from_utf8(bytes[..split].to_vec()).unwrap(),
                bytes[split + 4..].to_vec(),
            )
        }
    }

    impl Drop for WebFiles {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn serves_the_game_and_wasm_with_query_strings() {
        let files = WebFiles::new();
        let (headers, body) = files.request("GET", "/?version=1");
        assert!(headers.starts_with("HTTP/1.1 200"));
        assert!(headers.contains("Content-Type: text/html; charset=utf-8"));
        assert_eq!(body, b"<title>Dario</title>");

        let (headers, body) = files.request("GET", "/dario.wasm?version=1");
        assert!(headers.starts_with("HTTP/1.1 200"));
        assert!(headers.contains("Content-Type: application/wasm"));
        assert_eq!(body, b"\0asm\x01\0\0\0");
    }

    #[test]
    fn head_returns_wasm_metadata_without_a_body() {
        let files = WebFiles::new();
        let (headers, body) = files.request("HEAD", "/dario.wasm");
        assert!(headers.starts_with("HTTP/1.1 200"));
        assert!(headers.contains("Content-Type: application/wasm"));
        assert!(headers.contains("Content-Length: 8"));
        assert!(body.is_empty());
    }

    #[test]
    fn rejects_private_files_and_traversal() {
        let files = WebFiles::new();
        for path in [
            "/private.txt",
            "/../private.txt",
            "/%2e%2e/private.txt",
            "/vendor/../../private.txt",
            "/target/web/private.txt",
            "/vendor/",
        ] {
            let (headers, body) = files.request("GET", path);
            assert!(headers.starts_with("HTTP/1.1 404"), "{path}: {headers}");
            assert_eq!(body, b"Not found\n");
        }
    }

    #[test]
    fn rejects_writes_and_reports_missing_game_assets() {
        let files = WebFiles::new();
        let (headers, _) = files.request("POST", "/");
        assert!(headers.starts_with("HTTP/1.1 405"));
        assert!(headers.contains("Allow: GET, HEAD"));

        fs::remove_file(files.0.join("dario.wasm")).unwrap();
        let (headers, _) = files.request("GET", "/dario.wasm");
        assert!(headers.starts_with("HTTP/1.1 503"));
    }
}
