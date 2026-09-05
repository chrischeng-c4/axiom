use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{header, Method, StatusCode, Uri},
    routing::any,
    Router,
};
use service_http::{reverse_proxy_router, ReverseProxyPolicy, ReverseProxySelectionError};
use tower::ServiceExt;
use url::Url;

#[derive(Clone)]
struct FixedUpstream {
    base: Url,
    max_body: usize,
}

impl ReverseProxyPolicy for FixedUpstream {
    fn select_upstream(
        &self,
        _method: &Method,
        _uri: &Uri,
    ) -> Result<Url, ReverseProxySelectionError> {
        Ok(self.base.clone())
    }

    fn max_body_bytes(&self) -> usize {
        self.max_body
    }
}

#[tokio::test]
async fn proxy_preserves_route_and_end_to_end_headers_but_removes_hop_headers() {
    let upstream = Router::new().fallback(any(|request: Request| async move {
        let path = request.uri().path_and_query().unwrap().as_str().to_string();
        let kept = request
            .headers()
            .get("x-project")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();
        let hop = request.headers().contains_key(header::CONNECTION);
        let body = to_bytes(request.into_body(), 1_024).await.unwrap();
        (
            StatusCode::ACCEPTED,
            [("x-upstream", "yes")],
            format!("{path}|{kept}|{hop}|{}", String::from_utf8_lossy(&body)),
        )
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, upstream).await.unwrap() });

    let proxy = reverse_proxy_router(FixedUpstream {
        base: Url::parse(&format!("http://{address}")).unwrap(),
        max_body: 64,
    })
    .unwrap();
    let response = proxy
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs?source=test")
                .header("x-project", "project-a")
                .header(header::CONNECTION, "close")
                .body(Body::from("payload"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert_eq!(response.headers()["x-upstream"], "yes");
    let body = to_bytes(response.into_body(), 1_024).await.unwrap();
    assert_eq!(&body[..], b"/v1/logs?source=test|project-a|false|payload");
    server.abort();
}

#[tokio::test]
async fn body_limit_fails_before_contacting_the_upstream() {
    let proxy = reverse_proxy_router(FixedUpstream {
        base: Url::parse("http://127.0.0.1:9").unwrap(),
        max_body: 2,
    })
    .unwrap();
    let response = proxy
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .body(Body::from("too large"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_024).await.unwrap()).unwrap();
    assert_eq!(body["error"], "proxy_body_too_large");
}
