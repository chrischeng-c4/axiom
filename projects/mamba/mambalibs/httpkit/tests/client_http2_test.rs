use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use futures::StreamExt;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::header::{CONTENT_TYPE, HeaderValue};
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{Method, Request as HyperRequest, Response as HyperResponse, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use mambalibs_http::client::{HttpClient, HttpClientConfig};
use mambalibs_http::http::Request;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::{JoinHandle, JoinSet};

type ResponseBody = BoxBody<Bytes, Infallible>;

#[derive(Clone, Copy)]
enum ServerMode {
    Http1,
    Http2,
}

struct TestServer {
    addr: SocketAddr,
    stop: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

impl TestServer {
    async fn start(mode: ServerMode) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (stop_tx, mut stop_rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let mut conns = JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    accepted = listener.accept() => {
                        let Ok((stream, _)) = accepted else { break };
                        conns.spawn(async move {
                            let io = TokioIo::new(stream);
                            match mode {
                                ServerMode::Http1 => {
                                    let _ = http1::Builder::new()
                                        .serve_connection(io, service_fn(handle_http1))
                                        .await;
                                }
                                ServerMode::Http2 => {
                                    let _ = http2::Builder::new(TokioExecutor::new())
                                        .serve_connection(io, service_fn(handle_http2))
                                        .await;
                                }
                            }
                        });
                    }
                    Some(_) = conns.join_next() => {}
                }
            }
            conns.abort_all();
        });

        Self {
            addr,
            stop: Some(stop_tx),
            handle,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }

    async fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        let _ = self.handle.await;
    }
}

async fn handle_http1(
    req: HyperRequest<Incoming>,
) -> Result<HyperResponse<ResponseBody>, Infallible> {
    let body = serde_json::json!({
        "mode": "http1",
        "path": req.uri().path(),
        "seen_version": stable_version(req.version()),
    })
    .to_string();
    Ok(json_response(StatusCode::OK, body))
}

async fn handle_http2(
    req: HyperRequest<Incoming>,
) -> Result<HyperResponse<ResponseBody>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let seen_version = stable_version(req.version());

    match (method, path.as_str()) {
        (Method::GET, "/get") => {
            let body = serde_json::json!({
                "mode": "http2",
                "seen_version": seen_version,
            })
            .to_string();
            Ok(json_response(StatusCode::OK, body))
        }
        (Method::POST, "/echo") => {
            let header = req
                .headers()
                .get("x-custom")
                .and_then(|value| value.to_str().ok())
                .unwrap_or("")
                .to_string();
            let body_bytes = req.into_body().collect().await.unwrap().to_bytes();
            let body = serde_json::json!({
                "mode": "http2",
                "seen_version": seen_version,
                "x_custom": header,
                "body": String::from_utf8_lossy(&body_bytes).to_string(),
            })
            .to_string();
            Ok(json_response(StatusCode::OK, body))
        }
        (Method::GET, "/stream") => {
            let chunks = futures::stream::iter([
                Ok::<_, Infallible>(Frame::data(Bytes::from_static(b"chunk-a\n"))),
                Ok::<_, Infallible>(Frame::data(Bytes::from_static(b"chunk-b\n"))),
            ]);
            let mut response = HyperResponse::new(BodyExt::boxed(StreamBody::new(chunks)));
            response.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
            Ok(response)
        }
        _ => Ok(text_response(StatusCode::NOT_FOUND, "not found")),
    }
}

fn json_response(status: StatusCode, body: String) -> HyperResponse<ResponseBody> {
    let mut response = text_response(status, body);
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}

fn text_response(status: StatusCode, body: impl Into<Bytes>) -> HyperResponse<ResponseBody> {
    let mut response = HyperResponse::new(Full::new(body.into()).boxed());
    *response.status_mut() = status;
    response
}

fn stable_version(version: http::Version) -> &'static str {
    match version {
        http::Version::HTTP_09 => "HTTP/0.9",
        http::Version::HTTP_10 => "HTTP/1.0",
        http::Version::HTTP_11 => "HTTP/1.1",
        http::Version::HTTP_2 => "HTTP/2",
        http::Version::HTTP_3 => "HTTP/3",
        _ => "unknown",
    }
}

#[tokio::test]
async fn require_http2_get_exposes_protocol_metadata() {
    let server = TestServer::start(ServerMode::Http2).await;
    let client = HttpClient::new(HttpClientConfig::new().require_http2()).unwrap();

    let response = client.get(&server.url("/get")).await.unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(response.protocol_version(), "HTTP/2");
    assert!(response.is_http2());
    assert_eq!(response.json().unwrap()["seen_version"], "HTTP/2");

    server.stop().await;
}

#[tokio::test]
async fn require_http2_post_body_and_headers_work() {
    let server = TestServer::start(ServerMode::Http2).await;
    let client = HttpClient::new(HttpClientConfig::new().require_http2()).unwrap();

    let request = Request::post(server.url("/echo"))
        .header("x-custom", "from-test")
        .text("hello over h2");
    let response = client.send(request).await.unwrap();
    let json = response.json().unwrap();

    assert_eq!(response.protocol_version(), "HTTP/2");
    assert_eq!(json["seen_version"], "HTTP/2");
    assert_eq!(json["x_custom"], "from-test");
    assert_eq!(json["body"], "hello over h2");

    server.stop().await;
}

#[tokio::test]
async fn require_http2_streaming_response_yields_bytes() {
    let server = TestServer::start(ServerMode::Http2).await;
    let client = HttpClient::new(HttpClientConfig::new().require_http2()).unwrap();

    let mut stream = client
        .send_stream(Request::get(server.url("/stream")))
        .await
        .unwrap();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        body.extend_from_slice(&chunk.unwrap());
    }

    assert_eq!(body, b"chunk-a\nchunk-b\n");

    server.stop().await;
}

#[tokio::test]
async fn require_http2_fails_clearly_against_http1_only_server() {
    let server = TestServer::start(ServerMode::Http1).await;
    let client = HttpClient::new(HttpClientConfig::new().require_http2()).unwrap();

    let err = client.get(&server.url("/get")).await.unwrap_err();
    let msg = err.to_string();

    assert!(
        msg.contains("HTTP/2 required"),
        "strict HTTP/2 error should name the policy, got: {msg}"
    );

    server.stop().await;
}

#[tokio::test]
async fn auto_mode_preserves_http1_behavior() {
    let server = TestServer::start(ServerMode::Http1).await;
    let client = HttpClient::new(HttpClientConfig::new()).unwrap();

    let response = client.get(&server.url("/get")).await.unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(response.protocol_version(), "HTTP/1.1");
    assert_eq!(response.json().unwrap()["seen_version"], "HTTP/1.1");

    server.stop().await;
}
