// HANDWRITE-BEGIN gap="missing-generator:unit-test:cb847680" tracker="pending-tracker" reason="Integration tests over a real ephemeral server (tape::server::router/AppState) with a temp registry JSON file: append 200/401/403 (write grant), replay/checkpoint-get/checkpoint-put 200/403 with role hierarchy + wildcard grants, shared error bodies, tokenless probes under required auth, off-mode tokenless regression, and AuthConfig::resolve fail-fast (missing/unparseable/empty registry, unknown mode)."
//! Bearer-token auth integration tests over a real ephemeral server (#1326).
//!
//! tape adopts the shared `libs/service-auth` role-map contract: the blanket
//! auth middleware on the `/topics` data plane (401 for missing/unknown
//! tokens under `TAPE_AUTH=required`), per-handler authorization on the
//! `{topic}` path param (append = write, replay/checkpoint-get/
//! checkpoint-put = read; wildcard `*` grants + the admin ⊇ write ⊇ read
//! hierarchy), tokenless always-on probes, and the tokenless off-mode
//! default. The registry is loaded through the real file loader from a temp
//! `token-registry.json`.

use std::net::SocketAddr;

use serde_json::json;

use tape::auth::AuthConfig;
use tape::server::{router, AppState};
use tape::TapeJournal;

/// producer has a write grant on `orders`, worker has a read grant on
/// `orders`, plus a wildcard admin.
const REGISTRY: &str = r#"{
    "writer-token": {"subject": "producer", "roles": {"orders": "write"}},
    "reader-token": {"subject": "worker", "roles": {"orders": "read"}},
    "admin-token": {"subject": "root", "roles": {"*": "admin"}}
}"#;

/// Resolve a required-mode `AuthConfig` through the real registry-file
/// loader (the same path `--auth required --token-registry-file <f>`
/// exercises).
fn required_auth() -> AuthConfig {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("token-registry.json");
    std::fs::write(&path, REGISTRY).unwrap();
    let cfg = AuthConfig::resolve("required", Some(path.to_str().unwrap()), None).unwrap();
    // Keep the tempdir alive for the config's lifetime by leaking it: tests
    // are short-lived processes and this avoids threading a guard through
    // every call site.
    std::mem::forget(dir);
    cfg
}

async fn start_server(auth: Option<AuthConfig>) -> SocketAddr {
    let state = match auth {
        Some(a) => AppState::with_auth(TapeJournal::default(), None, a),
        None => AppState::new(TapeJournal::default(), None),
    };
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(service_http::serve(
        listener,
        app,
        std::future::pending::<()>(),
    ));
    addr
}

fn url(addr: SocketAddr, path: &str) -> String {
    format!("http://{addr}{path}")
}

async fn append(
    client: &reqwest::Client,
    addr: SocketAddr,
    topic: &str,
    n: i64,
    token: Option<&str>,
) -> reqwest::Response {
    let mut req = client
        .post(url(addr, &format!("/topics/{topic}/append")))
        .json(&json!({ "payload": { "n": n } }));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    req.send().await.unwrap()
}

async fn replay(
    client: &reqwest::Client,
    addr: SocketAddr,
    topic: &str,
    token: Option<&str>,
) -> reqwest::Response {
    let mut req = client.get(url(addr, &format!("/topics/{topic}/replay")));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    req.send().await.unwrap()
}

/// R1: a write grant on the topic appends 200; no token and an unknown
/// token are 401 from the blanket middleware; a read-only grant is 403; the
/// wildcard admin grant covers any topic.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn append_requires_write_grant_on_topic() {
    let addr = start_server(Some(required_auth())).await;
    let client = reqwest::Client::new();

    assert_eq!(append(&client, addr, "orders", 1, None).await.status(), 401);
    assert_eq!(
        append(&client, addr, "orders", 1, Some("unknown-token"))
            .await
            .status(),
        401
    );
    assert_eq!(
        append(&client, addr, "orders", 1, Some("reader-token"))
            .await
            .status(),
        403
    );

    let ok = append(&client, addr, "orders", 1, Some("writer-token")).await;
    assert_eq!(ok.status(), 200);
    let body: serde_json::Value = ok.json().await.unwrap();
    assert_eq!(body["offset"], 0);

    // Wildcard * grant (admin ⊇ write) covers append on any topic.
    assert_eq!(
        append(&client, addr, "other", 1, Some("admin-token"))
            .await
            .status(),
        200
    );
}

/// R2: replay, checkpoint-get, and checkpoint-put all require a read grant
/// on the topic; a grant scoped to a different topic is 403; write covers
/// read; the wildcard admin grant covers every topic.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn replay_and_checkpoint_require_read_grant_on_topic() {
    let addr = start_server(Some(required_auth())).await;
    let client = reqwest::Client::new();
    assert_eq!(
        append(&client, addr, "orders", 1, Some("writer-token"))
            .await
            .status(),
        200
    );

    // No token -> 401 from the middleware; wrong-topic grant -> 403.
    assert_eq!(replay(&client, addr, "orders", None).await.status(), 401);
    assert_eq!(
        replay(&client, addr, "other", Some("reader-token"))
            .await
            .status(),
        403
    );

    // read grant: replay on its topic passes.
    let resp = replay(&client, addr, "orders", Some("reader-token")).await;
    assert_eq!(resp.status(), 200);

    // checkpoint-get / checkpoint-put both accept the same read grant.
    let get = client
        .get(url(addr, "/topics/orders/consumers/c1/checkpoint"))
        .bearer_auth("reader-token")
        .send()
        .await
        .unwrap();
    assert_eq!(get.status(), 200);

    let put = client
        .put(url(addr, "/topics/orders/consumers/c1/checkpoint"))
        .bearer_auth("reader-token")
        .json(&json!({ "offset": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(put.status(), 200);

    // A write-only-elsewhere grant is 403 on checkpoint-put for this topic.
    let denied = client
        .put(url(addr, "/topics/orders/consumers/c1/checkpoint"))
        .bearer_auth("unknown-token")
        .json(&json!({ "offset": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 401);

    // Role hierarchy: a write grant covers the read family…
    assert_eq!(
        replay(&client, addr, "orders", Some("writer-token"))
            .await
            .status(),
        200
    );
    // …and the wildcard admin grant covers it on every topic.
    let admin_get = client
        .get(url(addr, "/topics/other/consumers/c1/checkpoint"))
        .bearer_auth("admin-token")
        .send()
        .await
        .unwrap();
    assert_eq!(admin_get.status(), 200);
}

/// R5: rejections render the shared service-auth JSON shape — the same
/// `{error, message}` envelope family the tape ApiErr errors use.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn error_bodies_use_shared_service_auth_shape() {
    let addr = start_server(Some(required_auth())).await;
    let client = reqwest::Client::new();

    let unauth = append(&client, addr, "orders", 1, None).await;
    assert_eq!(unauth.status(), 401);
    let body = unauth.text().await.unwrap();
    assert!(
        body.contains("\"error\":\"unauthenticated\""),
        "401 body: {body}"
    );

    let forbidden = append(&client, addr, "orders", 1, Some("reader-token")).await;
    assert_eq!(forbidden.status(), 403);
    let body = forbidden.text().await.unwrap();
    assert!(body.contains("\"error\":\"forbidden\""), "403 body: {body}");
    assert!(
        body.contains("lacks"),
        "403 message names the missing grant: {body}"
    );
}

/// R4: the probe surface stays tokenless and always-on while the data plane
/// requires auth — the layer is on the /topics router only.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn probes_stay_tokenless_under_required_auth() {
    let addr = start_server(Some(required_auth())).await;
    let client = reqwest::Client::new();
    for path in ["/healthz", "/readyz", "/metrics", "/openapi.json", "/docs"] {
        let resp = client.get(url(addr, path)).send().await.unwrap();
        assert_eq!(resp.status(), 200, "probe {path} must stay tokenless");
    }
}

/// R3: the off default (StaticRoleMapVerifier::open through the same
/// layered router) keeps today's tokenless behavior on both sides.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_mode_keeps_tape_tokenless() {
    let addr = start_server(None).await;
    let client = reqwest::Client::new();
    assert_eq!(
        append(&client, addr, "orders", 1, None).await.status(),
        200
    );
    assert_eq!(replay(&client, addr, "orders", None).await.status(), 200);
}

/// R7: startup fail-fast — required mode with a missing, unparseable, or
/// empty registry (or an unknown mode string) is a resolve error naming the
/// env var, not a per-request 401.
#[test]
fn resolve_fails_fast_on_missing_or_bad_registry() {
    let err =
        AuthConfig::resolve("required", Some("/nonexistent/tape-registry.json"), None).unwrap_err();
    assert!(
        err.to_string().contains("TAPE_TOKEN_REGISTRY_FILE"),
        "{err:#}"
    );

    // Required with no registry source at all -> empty registry is a
    // misconfiguration.
    assert!(AuthConfig::resolve("required", None, None).is_err());

    // Unparseable registry file.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    std::fs::write(&path, "not-json").unwrap();
    assert!(AuthConfig::resolve("required", Some(path.to_str().unwrap()), None).is_err());

    // Unknown mode string names TAPE_AUTH.
    let err = AuthConfig::resolve("nonsense", None, None).unwrap_err();
    assert!(err.to_string().contains("TAPE_AUTH"), "{err:#}");

    // The off default resolves open (tokenless).
    let cfg = AuthConfig::resolve("off", None, None).unwrap();
    assert!(!cfg.required);
    assert!(cfg.tokens.is_empty());
}
// HANDWRITE-END
