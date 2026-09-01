use std::time::Duration;

use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use service_backup::{AdminSnapshotRequest, AdminSnapshotTransport, AdminSnapshotTransportConfig};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[derive(Serialize)]
struct Claims<'a> {
    aud: [&'a str; 1],
    exp: u64,
}

fn token(audience: &str, key: &[u8]) -> String {
    encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &Claims {
            aud: [audience],
            exp: 4_102_444_800,
        },
        &EncodingKey::from_secret(key),
    )
    .unwrap()
}

fn transport(max_diagnostic_bytes: usize) -> AdminSnapshotTransport {
    AdminSnapshotTransport::with_config(AdminSnapshotTransportConfig {
        operation_timeout: Duration::from_secs(2),
        response_idle_timeout: Duration::from_secs(1),
        max_diagnostic_bytes,
        ..Default::default()
    })
    .unwrap()
}

#[tokio::test]
async fn projected_token_is_reread_and_product_headers_are_preserved_per_request() {
    let server = MockServer::start().await;
    let first = token("sift.axiom.dev", b"first");
    let second = token("sift.axiom.dev", b"second");
    for bearer in [&first, &second] {
        Mock::given(method("GET"))
            .and(path("/admin/backup"))
            .and(header("authorization", format!("Bearer {bearer}")))
            .and(header("x-sift-project", "project-a"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"snapshot".to_vec()))
            .expect(1)
            .mount(&server)
            .await;
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("token");
    std::fs::write(&path, &first).unwrap();
    let request = AdminSnapshotRequest::new()
        .with_projected_bearer(&path, "sift.axiom.dev")
        .with_header("x-sift-project", "project-a")
        .unwrap();
    let transport = transport(64);
    assert_eq!(
        transport.fetch(&server.uri(), &request).await.unwrap(),
        b"snapshot"
    );
    std::fs::write(&path, &second).unwrap();
    assert_eq!(
        transport.fetch(&server.uri(), &request).await.unwrap(),
        b"snapshot"
    );
}

#[tokio::test]
async fn non_success_diagnostic_is_bounded_and_structured() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/admin/backup"))
        .respond_with(ResponseTemplate::new(503).set_body_string("diagnostic-body-is-long"))
        .mount(&server)
        .await;
    let error = transport(10)
        .fetch(&server.uri(), &AdminSnapshotRequest::new())
        .await
        .unwrap_err();
    let diagnostic = error.diagnostic().expect("status error has diagnostics");
    assert_eq!(diagnostic.status, 503);
    assert_eq!(diagnostic.body.as_bytes().len(), 10);
    assert!(diagnostic.truncated);
    assert_eq!(diagnostic.operation.as_str(), "backup");
}
