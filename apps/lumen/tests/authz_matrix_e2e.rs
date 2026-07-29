// SPEC-MANAGED: apps/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Authorization matrix (TEST-STRATEGY security gate), phase 1 (#2871).
//!
//! The role dimension is gone with the registry: there is no way to be
//! authenticated-but-under-privileged until SubjectAccessReview lands
//! (#2869), so the 403 column of the old matrix cannot be produced. What
//! survives is the column that actually protects anything today — every
//! data-plane endpoint × {no credential, unknown credential} under a
//! `required` config must be 401, with no handler quietly skipping
//! `auth.ensure`. A route that answered 200 here would be a hole no later
//! phase would notice, because the later phases only ever add checks.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::storage::Engine;

fn required_server() -> TestServer {
    let engine = Arc::new(Engine::new());
    let auth = AuthConfig { required: true };
    TestServer::new(router(AppState::new(engine, Arc::new(auth)))).expect("server")
}

fn open_server() -> TestServer {
    let engine = Arc::new(Engine::new());
    TestServer::new(router(AppState::open(engine))).expect("server")
}

/// Every data-plane verb this file covers, as (method, path, body). Kept as
/// one list so a new endpoint is added to the gate in one place.
fn endpoints() -> Vec<(&'static str, &'static str, Option<Value>)> {
    let schema = json!({ "fields": { "email": { "type": "keyword" } } });
    let search =
        json!({ "query": { "term": { "field": "email", "value": "a@x.com" } }, "limit": 5 });
    let index = json!({ "items": [{ "external_id": "u1", "field": "email", "value": "a@x.com" }] });
    vec![
        ("GET", "/collections", None),
        ("PUT", "/collections/users", Some(schema)),
        ("POST", "/collections/users/search", Some(search)),
        ("POST", "/collections/users/index", Some(index)),
        ("GET", "/collections/users/stats", None),
        ("DELETE", "/collections/users", None),
    ]
}

async fn status(
    s: &TestServer,
    method: &str,
    path: &str,
    body: &Option<Value>,
    tok: Option<&str>,
) -> u16 {
    let mut r = match method {
        "GET" => s.get(path),
        "PUT" => s.put(path),
        "POST" => s.post(path),
        "DELETE" => s.delete(path),
        other => panic!("unhandled method {other}"),
    };
    if let Some(b) = body {
        r = r.json(b);
    }
    if let Some(t) = tok {
        r = r.add_header("authorization", format!("Bearer {t}"));
    }
    r.await.status_code().as_u16()
}

#[tokio::test]
async fn every_endpoint_is_401_under_required_auth_with_or_without_a_credential() {
    let s = required_server();
    for (method, path, body) in endpoints() {
        assert_eq!(
            status(&s, method, path, &body, None).await,
            401,
            "{method} {path} with no credential"
        );
        assert_eq!(
            status(&s, method, path, &body, Some("tok-admin")).await,
            401,
            "{method} {path} with an unknown credential"
        );
    }
}

/// The other half of the same gate: the same endpoint list must all answer on
/// an open server. Without this, the assertion above would still pass if a
/// route were removed or permanently broken — 401 and 404 are both "not 200".
#[tokio::test]
async fn every_endpoint_answers_on_an_open_server() {
    let s = open_server();
    for (method, path, body) in endpoints() {
        let code = status(&s, method, path, &body, None).await;
        assert!(
            (200..300).contains(&code),
            "{method} {path} on an open server returned {code}"
        );
    }
}
// CODEGEN-END
