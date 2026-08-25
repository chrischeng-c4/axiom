#![cfg(feature = "operator")]

//! Tests executing `fetch_capacity_catalog` against an in-process stub `kube::Client`.
//!
//! Exercises the success path and all four rejection sites across two shared `RejectionReason`
//! variants, distinguishing sites by their formatted error messages and asserting against
//! direct literal values.

use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use axum::http::{Request, Response, StatusCode};
use kube::client::Body;
use kube::Client;
use lumen::operator::capacity::{fetch_capacity_catalog, RejectionReason};
use lumen::operator::Lumen;
use serde_json::json;
use service_k8s::ManagedService;
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

fn recording_client(catalog_status: StatusCode) -> (Client, Arc<Mutex<Vec<(String, String)>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let seen = requests.clone();
    let service = service_fn(move |req: Request<Body>| {
        let seen = seen.clone();
        async move {
            let method = req.method().to_string();
            let path = req.uri().path().to_string();
            seen.lock().unwrap().push((method.clone(), path.clone()));
            let (status, body) = if method == "PATCH" {
                (
                    StatusCode::OK,
                    json!({
                        "apiVersion": "rbac.authorization.k8s.io/v1",
                        "kind": "ClusterRoleBinding",
                        "metadata": {"name": "auth-delegation"}
                    }),
                )
            } else {
                (
                    catalog_status,
                    json!({
                        "apiVersion": "v1",
                        "kind": "Status",
                        "status": "Failure",
                        "message": "configmaps \\\"lumen-capacity-catalog\\\" not found",
                        "reason": "NotFound",
                        "code": 404
                    }),
                )
            };
            let response = Response::builder()
                .status(status)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap();
            Ok::<_, Infallible>(response)
        }
    });
    (Client::new(service, "default"), requests)
}

fn reconcile_lumen(placement: serde_json::Value) -> Lumen {
    serde_json::from_value(json!({
        "apiVersion": "lumen.dev/v1alpha1",
        "kind": "Lumen",
        "metadata": {"name": "search", "namespace": "acme", "uid": "uid-1234"},
        "spec": {"image": "lumen:test", "placement": placement}
    }))
    .expect("valid test Lumen")
}

#[tokio::test]
async fn reconcile_plan_native_selector_skips_catalog_and_preserves_placement() {
    let (client, requests) = recording_client(StatusCode::NOT_FOUND);
    let expected_tolerations = json!([{
        "key": "dedicated",
        "operator": "Equal",
        "value": "lumen",
        "effect": "NoSchedule"
    }]);
    let lumen = reconcile_lumen(json!({
        "nodeSelector": {"cloud.google.com/gke-nodepool": "lumen-ssd"},
        "tolerations": expected_tolerations.clone()
    }));
    let plan = lumen.reconcile_plan(client).await.expect("native plan");

    let seen = requests.lock().unwrap().clone();
    assert!(seen.iter().any(|(method, _)| method == "PATCH"));
    assert!(
        !seen
            .iter()
            .any(|(method, path)| method == "GET" && path.contains("configmaps")),
        "native placement must not read the catalog: {seen:?}"
    );
    let statefulset = plan
        .children
        .iter()
        .find(|child| child["kind"] == "StatefulSet")
        .unwrap_or_else(|| {
            panic!(
                "native plan did not render a StatefulSet; kinds: {:?}",
                plan.children
                    .iter()
                    .map(|child| child["kind"].clone())
                    .collect::<Vec<_>>()
            )
        });
    let pod = &statefulset["spec"]["template"]["spec"];
    assert_eq!(
        pod["nodeSelector"],
        json!({"cloud.google.com/gke-nodepool": "lumen-ssd"})
    );
    assert_eq!(pod["tolerations"], expected_tolerations);
}

#[tokio::test]
async fn reconcile_plan_empty_selector_keeps_catalog_failure_on_legacy_path() {
    let (client, requests) = recording_client(StatusCode::NOT_FOUND);
    let lumen = reconcile_lumen(json!({}));
    let error = match lumen.reconcile_plan(client).await {
        Ok(_) => panic!("empty selector must require the catalog"),
        Err(error) => error,
    };
    let seen = requests.lock().unwrap().clone();
    assert!(seen.iter().any(|(method, _)| method == "PATCH"));
    assert!(
        seen.iter()
            .any(|(method, path)| method == "GET" && path.contains("configmaps")),
        "legacy path must GET the catalog: {seen:?}"
    );
    assert!(error
        .to_string()
        .contains("failed to read capacity catalog ConfigMap"));
}

#[tokio::test]
async fn reconcile_plan_tolerations_only_keeps_catalog_failure_on_legacy_path() {
    let (client, requests) = recording_client(StatusCode::NOT_FOUND);
    let lumen = reconcile_lumen(json!({
        "tolerations": [{
            "key": "dedicated",
            "operator": "Equal",
            "value": "lumen",
            "effect": "NoSchedule"
        }]
    }));
    let error = match lumen.reconcile_plan(client).await {
        Ok(_) => panic!("tolerations-only placement must require the catalog"),
        Err(error) => error,
    };
    let seen = requests.lock().unwrap().clone();
    assert!(seen.iter().any(|(method, _)| method == "PATCH"));
    assert!(
        seen.iter()
            .any(|(method, path)| method == "GET" && path.contains("configmaps")),
        "tolerations-only placement must GET the catalog: {seen:?}"
    );
    assert!(error
        .to_string()
        .contains("failed to read capacity catalog ConfigMap"));
}

#[tokio::test]
async fn reconcile_plan_non_default_machine_type_keeps_catalog_failure_on_legacy_path() {
    let (client, requests) = recording_client(StatusCode::NOT_FOUND);
    let lumen = reconcile_lumen(json!({
        "initialMachineType": "n2-standard-4",
        "nodeSelector": {"cloud.google.com/gke-nodepool": "lumen-ssd"}
    }));
    let error = match lumen.reconcile_plan(client).await {
        Ok(_) => panic!("non-default machine type must require the catalog"),
        Err(error) => error,
    };
    let seen = requests.lock().unwrap().clone();
    assert!(seen.iter().any(|(method, _)| method == "PATCH"));
    assert!(
        seen.iter()
            .any(|(method, path)| method == "GET" && path.contains("configmaps")),
        "non-default machine type must GET the catalog: {seen:?}"
    );
    assert!(error
        .to_string()
        .contains("failed to read capacity catalog ConfigMap"));
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
