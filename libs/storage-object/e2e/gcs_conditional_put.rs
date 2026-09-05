use storage_object::{GcsObjectStore, ObjectStore, PutCondition};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn gcs_put_maps_if_absent_to_generation_precondition() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/upload/storage/v1/b/test-bucket/o"))
        .and(query_param("uploadType", "media"))
        .and(query_param("name", "archive/segment-1"))
        .and(query_param("ifGenerationMatch", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "archive/segment-1",
            "size": "7",
            "generation": "42",
            "etag": "etag-42",
            "contentType": "application/octet-stream",
            "updated": "2026-08-31T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let endpoint = server.uri();
    let meta = tokio::task::spawn_blocking(move || {
        let store = GcsObjectStore::anonymous_emulator("test-bucket", "archive", endpoint).unwrap();
        store
            .put(
                "segment-1",
                b"payload",
                "application/octet-stream",
                PutCondition::IfAbsent,
            )
            .unwrap()
    })
    .await
    .unwrap();
    assert_eq!(meta.key, "segment-1");
    assert_eq!(meta.version.as_str(), "42");
}
