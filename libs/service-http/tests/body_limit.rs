// HANDWRITE-BEGIN gap="missing-generator:unit-test:9a6c1e40" tracker="#2484" reason="Live-router coverage for the shared body-limit layer: content-length-known rejection, under-limit pass-through, and streamed/chunked bodies without a declared length."
use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::routing::post;
use axum::Router;
use futures_util::stream;
use service_http::body_limit_layer;
use tower::ServiceExt as _;

const MAX_BYTES: usize = 16;

fn app() -> Router {
    Router::new()
        .route("/echo", post(echo))
        .layer(body_limit_layer(MAX_BYTES))
}

async fn echo(body: Bytes) -> String {
    body.len().to_string()
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    use http_body_util::BodyExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn under_limit_body_passes_through() {
    let payload = "a".repeat(MAX_BYTES - 1);
    let request = Request::builder()
        .method("POST")
        .uri("/echo")
        .header(header::CONTENT_LENGTH, payload.len())
        .body(Body::from(payload.clone()))
        .unwrap();

    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    use http_body_util::BodyExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], payload.len().to_string().as_bytes());
}

#[tokio::test(flavor = "current_thread")]
async fn at_limit_body_passes_through() {
    let payload = "a".repeat(MAX_BYTES);
    let request = Request::builder()
        .method("POST")
        .uri("/echo")
        .header(header::CONTENT_LENGTH, payload.len())
        .body(Body::from(payload))
        .unwrap();

    let response = app().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "exactly MAX_BYTES must not be rejected — the cap is inclusive"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn content_length_known_oversized_body_is_rejected_with_structured_413() {
    let payload = "a".repeat(MAX_BYTES + 1);
    let request = Request::builder()
        .method("POST")
        .uri("/echo")
        .header(header::CONTENT_LENGTH, payload.len())
        .body(Body::from(payload))
        .unwrap();

    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let body = json_body(response).await;
    assert_eq!(body["error"], "payload_too_large");
    assert!(body["message"].as_str().unwrap().contains("size limit"));
}

#[tokio::test(flavor = "current_thread")]
async fn streaming_body_without_content_length_over_cap_is_rejected() {
    // No Content-Length header — the same shape a chunked-transfer-encoded
    // client body arrives in. Multiple frames so the cap is actually crossed
    // mid-stream, not on the first read.
    let chunks: Vec<Result<Bytes, std::io::Error>> = vec![
        Ok(Bytes::from_static(b"aaaaaaaaaa")), // 10 bytes
        Ok(Bytes::from_static(b"aaaaaaaaaa")), // 20 bytes total, over MAX_BYTES=16
    ];
    let body = Body::from_stream(stream::iter(chunks));
    let request = Request::builder()
        .method("POST")
        .uri("/echo")
        .body(body)
        .unwrap();
    assert!(
        request.headers().get(header::CONTENT_LENGTH).is_none(),
        "precondition: this request must not declare Content-Length"
    );

    let response = app().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "a streamed body that crosses the cap mid-read must still be capped"
    );
    let body = json_body(response).await;
    assert_eq!(body["error"], "payload_too_large");
}

#[tokio::test(flavor = "current_thread")]
async fn streaming_body_without_content_length_under_cap_passes_through() {
    let chunks: Vec<Result<Bytes, std::io::Error>> = vec![
        Ok(Bytes::from_static(b"aaaaa")), // 5 bytes
        Ok(Bytes::from_static(b"aaaaa")), // 10 bytes total, under MAX_BYTES=16
    ];
    let body = Body::from_stream(stream::iter(chunks));
    let request = Request::builder()
        .method("POST")
        .uri("/echo")
        .body(body)
        .unwrap();

    let response = app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    use http_body_util::BodyExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], b"10");
}
// HANDWRITE-END
