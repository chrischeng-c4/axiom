#![cfg(feature = "operator")]

//! Tests executing `fetch_capacity_catalog` against an in-process stub `kube::Client`.
//!
//! Exercises the success path and all four rejection sites across two shared `RejectionReason`
//! variants, distinguishing sites by their formatted error messages and asserting against
//! direct literal values.

use std::convert::Infallible;

use axum::http::{Request, Response, StatusCode};
use kube::client::Body;
use kube::Client;
use lumen::operator::capacity::{fetch_capacity_catalog, RejectionReason};
use serde_json::json;
use tower::service_fn;

fn stub_client(status: StatusCode, body: serde_json::Value) -> Client {
    let body_bytes = serde_json::to_vec(&body).expect("serialize response json");
    let service = service_fn(move |_req: Request<Body>| {
        let body_bytes = body_bytes.clone();
        async move {
            let resp = Response::builder()
                .status(status)
                .header("content-type", "application/json")
                .body(Body::from(body_bytes))
                .expect("build response");
            Ok::<_, Infallible>(resp)
        }
    });
    Client::new(service, "default")
}

#[tokio::test]
async fn fetch_capacity_catalog_success() {
    let catalog_json_str = r#"{
      "version": "1.0.0",
      "entries": [
        {
          "machine_type": "c2-standard-16",
          "selector": "lumen.axiom.dev/capacity-profile=c2-standard-16",
          "stable_selector": {
            "key": "lumen.axiom.dev/capacity-profile",
            "value": "c2-standard-16"
          },
          "max_nodes": 42,
          "min_nodes": 0,
          "lifecycle_state": "ready",
          "pool_group": "lumen-data"
        }
      ]
    }"#;

    let cm_response = json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "custom-catalog-cm",
            "namespace": "custom-catalog-ns",
        },
        "data": {
            "catalog.json": catalog_json_str,
        },
    });

    let client = stub_client(StatusCode::OK, cm_response);
    let catalog = fetch_capacity_catalog(&client, "custom-catalog-ns", "custom-catalog-cm")
        .await
        .expect("fetch_capacity_catalog should succeed");

    assert_eq!(catalog.version, "1.0.0");
    assert_eq!(catalog.entries.len(), 1);
    assert_eq!(catalog.entries[0].machine_type, "c2-standard-16");
    assert_eq!(catalog.entries[0].max_nodes, 42);
    assert_eq!(
        catalog.entries[0].selector,
        "lumen.axiom.dev/capacity-profile=c2-standard-16"
    );
    assert_eq!(
        catalog.entries[0].stable_selector.key,
        "lumen.axiom.dev/capacity-profile"
    );
    assert_eq!(catalog.entries[0].stable_selector.value, "c2-standard-16");
    assert_eq!(catalog.entries[0].lifecycle_state, "ready");
}

#[tokio::test]
async fn fetch_capacity_catalog_missing_configmap_rejected() {
    let error_response = json!({
        "apiVersion": "v1",
        "kind": "Status",
        "status": "Failure",
        "message": "configmaps \"missing-cm\" not found",
        "reason": "NotFound",
        "code": 404,
    });

    let client = stub_client(StatusCode::NOT_FOUND, error_response);
    let rejection = fetch_capacity_catalog(&client, "missing-ns", "missing-cm")
        .await
        .expect_err("fetch_capacity_catalog should fail when configmap is missing");

    assert_eq!(rejection.reason, RejectionReason::CatalogMissing);
    assert_eq!(rejection.field_path, "catalog");
    assert!(
        rejection
            .message
            .contains("failed to read capacity catalog ConfigMap"),
        "expected message to contain 'failed to read capacity catalog ConfigMap', got: {}",
        rejection.message
    );
    assert!(
        rejection.message.contains("missing-ns/missing-cm"),
        "expected message to contain 'missing-ns/missing-cm', got: {}",
        rejection.message
    );
}

#[tokio::test]
async fn fetch_capacity_catalog_no_data_rejected() {
    let cm_response = json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "nodata-cm",
            "namespace": "nodata-ns",
        },
    });

    let client = stub_client(StatusCode::OK, cm_response);
    let rejection = fetch_capacity_catalog(&client, "nodata-ns", "nodata-cm")
        .await
        .expect_err("fetch_capacity_catalog should fail when configmap has no data");

    assert_eq!(rejection.reason, RejectionReason::CatalogMissing);
    assert_eq!(rejection.field_path, "catalog");
    assert!(
        rejection.message.contains("has no data"),
        "expected message to contain 'has no data', got: {}",
        rejection.message
    );
    assert_eq!(
        rejection.message,
        "ConfigMap `nodata-ns/nodata-cm` has no data"
    );
}

#[tokio::test]
async fn fetch_capacity_catalog_missing_catalog_json_key_rejected() {
    let cm_response = json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "badkey-cm",
            "namespace": "badkey-ns",
        },
        "data": {
            "other_config.json": "{\"version\":\"1.0.0\"}",
        },
    });

    let client = stub_client(StatusCode::OK, cm_response);
    let rejection = fetch_capacity_catalog(&client, "badkey-ns", "badkey-cm")
        .await
        .expect_err("fetch_capacity_catalog should fail when configmap is missing catalog.json key");

    assert_eq!(rejection.reason, RejectionReason::CatalogIncompatible);
    assert_eq!(rejection.field_path, "catalog");
    assert!(
        rejection.message.contains("missing `catalog.json` key"),
        "expected message to contain 'missing `catalog.json` key', got: {}",
        rejection.message
    );
    assert_eq!(
        rejection.message,
        "ConfigMap `badkey-ns/badkey-cm` missing `catalog.json` key"
    );
}

#[tokio::test]
async fn fetch_capacity_catalog_malformed_json_rejected() {
    let cm_response = json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": "malformed-cm",
            "namespace": "malformed-ns",
        },
        "data": {
            "catalog.json": "{ not valid json }",
        },
    });

    let client = stub_client(StatusCode::OK, cm_response);
    let rejection = fetch_capacity_catalog(&client, "malformed-ns", "malformed-cm")
        .await
        .expect_err("fetch_capacity_catalog should fail when catalog.json is malformed");

    assert_eq!(rejection.reason, RejectionReason::CatalogIncompatible);
    assert_eq!(rejection.field_path, "catalog");
    assert!(
        rejection.message.contains("failed to parse `catalog.json`"),
        "expected message to contain 'failed to parse `catalog.json`', got: {}",
        rejection.message
    );
    assert!(
        rejection.message.contains("malformed-ns/malformed-cm"),
        "expected message to contain 'malformed-ns/malformed-cm', got: {}",
        rejection.message
    );
}
