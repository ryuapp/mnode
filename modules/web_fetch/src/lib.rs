use utils::add_internal_function;
use rquickjs::Ctx;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    ctx.eval::<(), _>(include_str!("fetch.js"))
}

type FetchResult = Option<Result<String, String>>;

struct FetchState {
    next_id: u64,
    pending: HashMap<u64, Arc<Mutex<FetchResult>>>,
}

static RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> = once_cell::sync::Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

static FETCH_STATE: once_cell::sync::Lazy<Mutex<FetchState>> = once_cell::sync::Lazy::new(|| {
    Mutex::new(FetchState {
        next_id: 0,
        pending: HashMap::new(),
    })
});

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn Error>> {
    ctx.eval::<(), _>("globalThis[Symbol.for('mnode.internal')].fetch = {};")?;

    add_internal_function!(ctx, "fetch.start", |url: String,
                                                method: String,
                                                headers: String,
                                                body: String|
     -> u64 {
        fetch_start(url, method, headers, body)
    });

    add_internal_function!(ctx, "fetch.poll", |id: u64| -> String { fetch_poll(id) });

    Ok(())
}

fn fetch_start(url: String, method: String, headers: String, body: String) -> u64 {
    let mut state = FETCH_STATE.lock().unwrap();
    let id = state.next_id;
    state.next_id += 1;

    let result = Arc::new(Mutex::new(None));
    state.pending.insert(id, result.clone());

    RUNTIME.spawn(async move {
        let res = fetch_request(url, method, headers, body).await;
        *result.lock().unwrap() = Some(res);
    });

    id
}

fn fetch_poll(id: u64) -> String {
    let mut state = FETCH_STATE.lock().unwrap();
    if let Some(result_arc) = state.pending.get(&id) {
        let mut result = result_arc.lock().unwrap();
        if let Some(res) = result.take() {
            // Remove from pending map
            drop(result);
            state.pending.remove(&id);

            match res {
                Ok(json) => json,
                Err(e) => format!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\"")),
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

async fn fetch_request(
    url: String,
    method: String,
    _headers: String,
    _body: String,
) -> Result<String, String> {
    // Parse URL
    let uri: hyper::Uri = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;

    // Get host for Host header
    let host = uri.host().ok_or("Missing host in URL")?;
    let host_header = if let Some(port) = uri.port_u16() {
        format!("{}:{}", host, port)
    } else {
        host.to_string()
    };

    // Create request
    let req = hyper::Request::builder()
        .method(method.as_str())
        .uri(&uri)
        .header("Host", host_header)
        .header("User-Agent", "mnode/0.1")
        .body(http_body_util::Empty::<bytes::Bytes>::new())
        .map_err(|e| format!("Failed to build request: {}", e))?;

    // Connect and send request
    let response = fetch_impl(req).await?;

    // Read response body
    let status = response.status().as_u16();
    let headers = format!("{:?}", response.headers());

    use http_body_util::BodyExt;

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?
        .to_bytes();

    let body_text = String::from_utf8_lossy(&body_bytes).to_string();

    // Properly escape the body text for JSON
    let escaped_body = body_text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                format!("\\u{:04x}", c as u32)
            } else {
                c.to_string()
            }
        })
        .collect::<String>();

    Ok(format!(
        r#"{{"status":{},"headers":{},"body":"{}"}}"#,
        status,
        serde_json::to_string(&headers).unwrap(),
        escaped_body
    ))
}

#[cfg(feature = "native-tls")]
mod tls {
    use tokio::net::TcpStream;

    pub enum IoStream {
        Plain(TcpStream),
        Tls(tokio_native_tls::TlsStream<TcpStream>),
    }

    impl tokio::io::AsyncRead for IoStream {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_read(cx, buf),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            }
        }
    }

    impl tokio::io::AsyncWrite for IoStream {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_write(cx, buf),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            }
        }

        fn poll_flush(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_flush(cx),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_flush(cx),
            }
        }

        fn poll_shutdown(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_shutdown(cx),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            }
        }
    }

    pub async fn create_tls_stream(host: &str, port: u16) -> Result<IoStream, String> {
        use tokio::net::TcpStream;

        let stream = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        let connector = tokio_native_tls::native_tls::TlsConnector::new()
            .map_err(|e| format!("TLS connector creation failed: {}", e))?;
        let connector = tokio_native_tls::TlsConnector::from(connector);
        let tls_stream = connector
            .connect(host, stream)
            .await
            .map_err(|e| format!("TLS handshake failed: {}", e))?;

        Ok(IoStream::Tls(tls_stream))
    }
}

#[cfg(feature = "rustls")]
mod tls {
    use std::sync::Arc;
    use tokio::net::TcpStream;

    pub enum IoStream {
        Plain(TcpStream),
        Tls(tokio_rustls::client::TlsStream<TcpStream>),
    }

    impl tokio::io::AsyncRead for IoStream {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_read(cx, buf),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            }
        }
    }

    impl tokio::io::AsyncWrite for IoStream {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_write(cx, buf),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            }
        }

        fn poll_flush(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_flush(cx),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_flush(cx),
            }
        }

        fn poll_shutdown(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut *self {
                IoStream::Plain(s) => std::pin::Pin::new(s).poll_shutdown(cx),
                IoStream::Tls(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            }
        }
    }

    pub async fn create_tls_stream(host: &str, port: u16) -> Result<IoStream, String> {
        use tokio::net::TcpStream;
        use tokio_rustls::rustls;

        let stream = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        let root_store = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect(),
        };

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
        let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
            .map_err(|e| format!("Invalid server name: {}", e))?;

        let tls_stream = connector
            .connect(server_name, stream)
            .await
            .map_err(|e| format!("TLS handshake failed: {}", e))?;

        Ok(IoStream::Tls(tls_stream))
    }
}

use tls::IoStream;

async fn fetch_impl(
    req: hyper::Request<http_body_util::Empty<bytes::Bytes>>,
) -> Result<hyper::Response<hyper::body::Incoming>, String> {
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpStream;

    let host = req.uri().host().ok_or("Missing host")?;
    let scheme = req.uri().scheme_str().unwrap_or("http");

    // Determine port and create appropriate stream
    let io = match scheme {
        "https" => {
            let port = req.uri().port_u16().unwrap_or(443);

            // Resolve host to avoid IPv6 fallback delays
            let tls_host = if host == "localhost" {
                "127.0.0.1"
            } else {
                host
            };

            let io_stream = tls::create_tls_stream(tls_host, port).await?;
            TokioIo::new(io_stream)
        }
        "http" => {
            let port = req.uri().port_u16().unwrap_or(80);

            // Resolve host to avoid IPv6 fallback delays
            let addr = if host == "localhost" {
                format!("127.0.0.1:{}", port)
            } else {
                format!("{}:{}", host, port)
            };

            let stream = TcpStream::connect(&addr)
                .await
                .map_err(|e| format!("Connection failed: {}", e))?;

            TokioIo::new(IoStream::Plain(stream))
        }
        scheme => return Err(format!("Unsupported scheme: {}", scheme)),
    };

    // Establish HTTP connection
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("Handshake failed: {}", e))?;

    // Spawn connection task
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Connection error: {:?}", e);
        }
    });

    // Send request
    sender
        .send_request(req)
        .await
        .map_err(|e| format!("Request failed: {}", e))
}
