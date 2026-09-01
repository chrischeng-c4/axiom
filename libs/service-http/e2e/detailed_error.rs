use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};
use http_body_util::BodyExt;
use service_http::{ApiErr, ProjectionMetadata};

#[tokio::test]
async fn existing_basic_error_wire_stays_exact() {
    let response = ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", "bad input").into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(response.headers().get(header::RETRY_AFTER).is_none());

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
        serde_json::json!({"error": "bad_request", "message": "bad input"})
    );
}

#[tokio::test]
async fn detailed_error_keeps_retry_header_and_projection_body_in_sync() {
    let response = ApiErr::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "projection_lag",
        "logging projection is behind",
    )
    .with_retryable(true)
    .with_retry_after_seconds(3)
    .with_projection(ProjectionMetadata {
        projection: "logging-store".into(),
        required_cursor: 42,
        current_cursor: 39,
    })
    .into_response();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[header::RETRY_AFTER], "3");
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
        serde_json::json!({
            "error": "projection_lag",
            "message": "logging projection is behind",
            "retryable": true,
            "projection": "logging-store",
            "required_cursor": 42,
            "current_cursor": 39,
            "retry_after_seconds": 3
        })
    );
}

#[tokio::test]
async fn detailed_non_retryable_error_has_no_retry_header() {
    let response = ApiErr::new(StatusCode::FORBIDDEN, "forbidden", "denied")
        .with_retryable(false)
        .into_response();

    assert!(response.headers().get(header::RETRY_AFTER).is_none());
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
        serde_json::json!({
            "error": "forbidden",
            "message": "denied",
            "retryable": false
        })
    );
}
