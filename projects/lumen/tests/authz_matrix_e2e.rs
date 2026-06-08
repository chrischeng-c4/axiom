//! Authorization matrix (TEST-STRATEGY security gate): every data-plane
//! endpoint × {no token, wrong-scope token, right-scope token} returns the
//! correct 401 / 403 / 200, and no handler silently skips `auth.ensure`.

use std::collections::HashMap;
use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::auth::{AuthConfig, Role, TokenClaims};
use lumen::storage::Engine;

const READER: &str = "tok-reader"; // Read on "users"
const WRITER: &str = "tok-writer"; // Write on "users"
const ADMIN: &str = "tok-admin"; //  Admin on "*"

fn claims(roles: &[(&str, Role)]) -> TokenClaims {
    TokenClaims {
        subject: "s".into(),
        roles: roles.iter().map(|(c, r)| (c.to_string(), *r)).collect(),
    }
}

fn auth_server() -> TestServer {
    let mut tokens = HashMap::new();
    tokens.insert(READER.to_string(), claims(&[("users", Role::Read)]));
    tokens.insert(WRITER.to_string(), claims(&[("users", Role::Write)]));
    tokens.insert(ADMIN.to_string(), claims(&[("*", Role::Admin)]));
    let auth = AuthConfig {
        required: true,
        tokens,
    };
    let engine = Arc::new(Engine::new());
    TestServer::new(router(AppState::new(engine, Arc::new(auth)))).expect("server")
}

/// POST `path` with `body`, optionally bearer-authenticated → HTTP status.
async fn post(s: &TestServer, path: &str, body: &Value, tok: Option<&str>) -> u16 {
    let mut r = s.post(path).json(body);
    if let Some(t) = tok {
        r = r.add_header("authorization", format!("Bearer {t}"));
    }
    r.await.status_code().as_u16()
}

async fn get(s: &TestServer, path: &str, tok: Option<&str>) -> u16 {
    let mut r = s.get(path);
    if let Some(t) = tok {
        r = r.add_header("authorization", format!("Bearer {t}"));
    }
    r.await.status_code().as_u16()
}

async fn put(s: &TestServer, path: &str, body: &Value, tok: Option<&str>) -> u16 {
    let mut r = s.put(path).json(body);
    if let Some(t) = tok {
        r = r.add_header("authorization", format!("Bearer {t}"));
    }
    r.await.status_code().as_u16()
}

#[tokio::test]
async fn authz_matrix_enforced_on_every_endpoint() {
    let s = auth_server();
    let schema = json!({ "fields": { "email": { "type": "keyword" } } });
    let search =
        json!({ "query": { "term": { "field": "email", "value": "a@x.com" } }, "limit": 5 });
    let index = json!({ "items": [{ "external_id": "u1", "field": "email", "value": "a@x.com" }] });

    // create = Admin: seed "users".
    assert_eq!(
        put(&s, "/collections/users", &schema, Some(ADMIN)).await,
        200
    );

    // No token → 401 on every endpoint (no handler skips auth).
    assert_eq!(
        post(&s, "/collections/users/search", &search, None).await,
        401
    );
    assert_eq!(get(&s, "/collections/users/stats", None).await, 401);
    assert_eq!(
        post(&s, "/collections/users/index", &index, None).await,
        401
    );
    assert_eq!(put(&s, "/collections/users", &schema, None).await, 401);

    // Bogus token → 401.
    assert_eq!(
        post(&s, "/collections/users/search", &search, Some("nope")).await,
        401
    );

    // Read endpoints (search, stats): reader / writer / admin all 200.
    for t in [READER, WRITER, ADMIN] {
        assert_eq!(
            post(&s, "/collections/users/search", &search, Some(t)).await,
            200,
            "search/{t}"
        );
        assert_eq!(
            get(&s, "/collections/users/stats", Some(t)).await,
            200,
            "stats/{t}"
        );
    }

    // Write endpoint (index): reader 403, writer/admin 200.
    assert_eq!(
        post(&s, "/collections/users/index", &index, Some(READER)).await,
        403,
        "index/reader"
    );
    assert_eq!(
        post(&s, "/collections/users/index", &index, Some(WRITER)).await,
        200,
        "index/writer"
    );
    assert_eq!(
        post(&s, "/collections/users/index", &index, Some(ADMIN)).await,
        200,
        "index/admin"
    );

    // Admin endpoint (create new collection): reader/writer 403, admin 200.
    assert_eq!(
        put(&s, "/collections/other", &schema, Some(READER)).await,
        403,
        "create/reader"
    );
    assert_eq!(
        put(&s, "/collections/other", &schema, Some(WRITER)).await,
        403,
        "create/writer"
    );
    assert_eq!(
        put(&s, "/collections/other", &schema, Some(ADMIN)).await,
        200,
        "create/admin"
    );
}
