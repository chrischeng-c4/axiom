// HANDWRITE-BEGIN gap="missing-generator:unit-test:615ff6fb" tracker="pending-tracker" reason="Integration tests over a real ephemeral server with a temp registry JSON file: publish 200/401/403 (write grant), consume-side lease/ack/len 200/403 with role hierarchy + wildcard grants, streaming consume 401/403, shared error bodies, tokenless probes under required auth, off-mode tokenless regression, and AuthConfig::resolve fail-fast (missing/unparseable/empty registry, unknown mode)."
//! Bearer-token auth integration tests over a real ephemeral server (#1206).
//!
//! relay adopts the shared `libs/service-auth` role-map contract: the blanket
//! auth middleware on the `/v1` data plane (401 for missing/unknown tokens
//! under `RELAY_AUTH=required`), per-handler-group authorization on the
//! `{subject}` path param (publish family = write, consume family = read;
//! wildcard `*` grants + the admin ⊇ write ⊇ read hierarchy), tokenless
//! always-on probes, and the tokenless off-mode default. The registry is
//! loaded through the real file loader from a temp `token-registry.json`.

use std::net::SocketAddr;

use serde_json::json;

use relay::auth::AuthConfig;
use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;

/// jobs-scoped writer + reader, plus a wildcard admin.
const REGISTRY: &str = r#"{
    "writer-token": {"subject": "producer", "roles": {"jobs": "write"}},
    "reader-token": {"subject": "worker", "roles": {"jobs": "read"}},
    "admin-token": {"subject": "root", "roles": {"*": "admin"}}
}"#;

/// Resolve a required-mode `AuthConfig` through the real registry-file loader
/// (the same path `--auth required --token-registry-file <f>` exercises).
fn required_auth() -> AuthConfig {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("token-registry.json");
    std::fs::write(&path, REGISTRY).unwrap();
    AuthConfig::resolve("required", Some(path.to_str().unwrap()), None).unwrap()
}

async fn start_server(auth: Option<AuthConfig>) -> SocketAddr {
    let config = RelayServerConfig::ephemeral();
    let state = match auth {
        Some(a) => AppState::with_auth(config, a),
        None => AppState::new(config),
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

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

fn url(addr: SocketAddr, path: &str) -> String {
    format!("http://{addr}{path}")
}

async fn publish(
    client: &reqwest::Client,
    addr: SocketAddr,
    subject: &str,
    id: &str,
    token: Option<&str>,
) -> reqwest::Response {
    let mut req = client
        .post(url(addr, &format!("/v1/{subject}/publish")))
        .json(&json!({ "message_id": id, "payload": { "n": id } }));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    req.send().await.unwrap()
}

async fn lease(
    client: &reqwest::Client,
    addr: SocketAddr,
    subject: &str,
    token: Option<&str>,
) -> reqwest::Response {
    let mut req = client
        .post(url(addr, &format!("/v1/{subject}/lease")))
        .json(&json!({ "consumer_id": "c1" }));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    req.send().await.unwrap()
}

/// R1/AC1 (publish side): a write grant on the subject publishes 200; no
/// token and an unknown token are 401 from the blanket middleware; a
/// read-only grant is 403; the wildcard admin grant covers any subject.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn publish_requires_write_grant_on_subject() {
    let addr = start_server(Some(required_auth())).await;
    let client = h2c_client();

    assert_eq!(
        publish(&client, addr, "jobs", "m0", None).await.status(),
        401
    );
    assert_eq!(
        publish(&client, addr, "jobs", "m0", Some("unknown-token"))
            .await
            .status(),
        401
    );
    assert_eq!(
        publish(&client, addr, "jobs", "m0", Some("reader-token"))
            .await
            .status(),
        403
    );

    let ok = publish(&client, addr, "jobs", "m0", Some("writer-token")).await;
    assert_eq!(ok.status(), 200);
    let body: serde_json::Value = ok.json().await.unwrap();
    assert_eq!(body["seq"], 0);

    // Wildcard * grant (admin ⊇ write) covers publish on any subject.
    assert_eq!(
        publish(&client, addr, "other", "m0", Some("admin-token"))
            .await
            .status(),
        200
    );
    // publish-batch sits in the same write group: read-only grant is 403.
    let batch = client
        .post(url(addr, "/v1/jobs/publish-batch"))
        .bearer_auth("reader-token")
        .json(&json!({ "messages": [{ "message_id": "b0", "payload": {} }] }))
        .send()
        .await
        .unwrap();
    assert_eq!(batch.status(), 403);
}

/// R3/AC1 (consume side): read grants lease/ack/len on their subject; a grant
/// scoped to another subject is 403; write covers read; the wildcard admin
/// grant covers the read family everywhere (heartbeat, lease-batch).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consume_side_requires_read_grant_on_subject() {
    let addr = start_server(Some(required_auth())).await;
    let client = h2c_client();
    assert_eq!(
        publish(&client, addr, "jobs", "m0", Some("writer-token"))
            .await
            .status(),
        200
    );

    // No token → 401 from the middleware; wrong-subject grant → 403.
    assert_eq!(lease(&client, addr, "jobs", None).await.status(), 401);
    assert_eq!(
        lease(&client, addr, "other", Some("reader-token"))
            .await
            .status(),
        403
    );

    // read grant: lease, ack, len on its subject all pass.
    let resp = lease(&client, addr, "jobs", Some("reader-token")).await;
    assert_eq!(resp.status(), 200);
    let leased: serde_json::Value = resp.json().await.unwrap();
    let lease_id = leased["lease"]["lease_id"].as_str().unwrap().to_string();
    let epoch = leased["lease"]["epoch"].as_u64().unwrap();

    let ack = client
        .post(url(addr, "/v1/jobs/ack"))
        .bearer_auth("reader-token")
        .json(&json!({ "lease_id": lease_id, "epoch": epoch }))
        .send()
        .await
        .unwrap();
    assert_eq!(ack.status(), 200);
    let acked: serde_json::Value = ack.json().await.unwrap();
    assert_eq!(acked["acked"], true);

    let len = client
        .get(url(addr, "/v1/jobs/len"))
        .bearer_auth("reader-token")
        .send()
        .await
        .unwrap();
    assert_eq!(len.status(), 200);

    // Role hierarchy: a write grant covers the read family…
    assert_eq!(
        lease(&client, addr, "jobs", Some("writer-token"))
            .await
            .status(),
        200
    );
    // …and the wildcard admin grant covers it on every subject.
    let hb = client
        .post(url(addr, "/v1/jobs/heartbeat"))
        .bearer_auth("admin-token")
        .json(&json!({ "lease_id": "missing", "epoch": 0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(hb.status(), 200);
    let lb = client
        .post(url(addr, "/v1/jobs/lease-batch"))
        .bearer_auth("reader-token")
        .json(&json!({ "consumer_id": "c1", "max": 4 }))
        .send()
        .await
        .unwrap();
    assert_eq!(lb.status(), 200);
}

/// R3: the streaming consume path enforces the same contract — 401 without a
/// token (middleware), 403 before the Subscribe handshake when the token's
/// grants do not cover read on the subject.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn streaming_consume_enforces_read_grant() {
    let addr = start_server(Some(required_auth())).await;
    let client = h2c_client();

    let resp = client
        .post(url(addr, "/v1/jobs/consume"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    let resp = client
        .post(url(addr, "/v1/other/consume"))
        .bearer_auth("reader-token")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

/// R4: rejections render the shared service-auth JSON shape — the same
/// `{error, message}` envelope family the #1205 ApiErr errors use.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn error_bodies_use_shared_service_auth_shape() {
    let addr = start_server(Some(required_auth())).await;
    let client = h2c_client();

    let unauth = publish(&client, addr, "jobs", "m0", None).await;
    assert_eq!(unauth.status(), 401);
    let body = unauth.text().await.unwrap();
    assert!(
        body.contains("\"error\":\"unauthenticated\""),
        "401 body: {body}"
    );

    let forbidden = publish(&client, addr, "jobs", "m0", Some("reader-token")).await;
    assert_eq!(forbidden.status(), 403);
    let body = forbidden.text().await.unwrap();
    assert!(body.contains("\"error\":\"forbidden\""), "403 body: {body}");
    assert!(
        body.contains("lacks"),
        "403 message names the missing grant: {body}"
    );
}

/// R6/AC2: the probe surface stays tokenless and always-on while the data
/// plane requires auth — the layer is on the /v1 router only.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn probes_stay_tokenless_under_required_auth() {
    let addr = start_server(Some(required_auth())).await;
    let client = h2c_client();
    for path in ["/healthz", "/readyz", "/metrics", "/openapi.json", "/docs"] {
        let resp = client.get(url(addr, path)).send().await.unwrap();
        assert_eq!(resp.status(), 200, "probe {path} must stay tokenless");
    }
}

/// R2/AC3: the off default (StaticRoleMapVerifier::open through the same
/// layered router) keeps today's tokenless behavior on both sides.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn off_mode_keeps_tokenless_behavior() {
    let addr = start_server(None).await;
    let client = h2c_client();
    assert_eq!(
        publish(&client, addr, "jobs", "m0", None).await.status(),
        200
    );
    assert_eq!(lease(&client, addr, "jobs", None).await.status(), 200);
}

/// R2/AC4: startup fail-fast — required mode with a missing, unparseable, or
/// empty registry (or an unknown mode string) is a resolve error naming the
/// env var, not a per-request 401.
#[test]
fn resolve_fails_fast_on_missing_or_bad_registry() {
    let err = AuthConfig::resolve("required", Some("/nonexistent/relay-registry.json"), None)
        .unwrap_err();
    assert!(
        err.to_string().contains("RELAY_TOKEN_REGISTRY_FILE"),
        "{err:#}"
    );

    // Required with no registry source at all → empty registry is a
    // misconfiguration.
    assert!(AuthConfig::resolve("required", None, None).is_err());

    // Unparseable registry file.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    std::fs::write(&path, "not-json").unwrap();
    assert!(AuthConfig::resolve("required", Some(path.to_str().unwrap()), None).is_err());

    // Unknown mode string names RELAY_AUTH.
    let err = AuthConfig::resolve("nonsense", None, None).unwrap_err();
    assert!(err.to_string().contains("RELAY_AUTH"), "{err:#}");

    // The off default resolves open (tokenless).
    let cfg = AuthConfig::resolve("off", None, None).unwrap();
    assert!(!cfg.required);
    assert!(cfg.tokens.is_empty());
}
// HANDWRITE-END
