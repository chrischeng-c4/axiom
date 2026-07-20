// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-service-auth" tracker="#766" reason="Defer integration proof for shared credential rotation and queue-scoped authorization."
use axum::http::{header, HeaderMap};
use defer::AuthConfig;
use service_auth::{Role, Verifier};

const REGISTRY: &str = r#"{
    "writer-token": {"subject": "producer", "roles": {"jobs": "write"}},
    "reader-token": {"subject": "worker", "roles": {"jobs": "read"}},
    "admin-token": {"subject": "root", "roles": {"*": "admin"}}
}"#;

fn required_auth() -> AuthConfig {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("token-registry.json");
    std::fs::write(&path, REGISTRY).unwrap();
    AuthConfig::resolve("required", Some(path.to_str().unwrap()), None).unwrap()
}

#[test]
fn defer_auth_adapter_rotates_the_shared_registry_without_restart() {
    let verifier = required_auth().verifier();
    let mut before = HeaderMap::new();
    before.insert(
        header::AUTHORIZATION,
        "Bearer writer-token".parse().unwrap(),
    );
    let principal = verifier.authenticate(&before).unwrap();
    assert!(principal.ensure("jobs", Role::Write).is_ok());

    verifier
        .reload_json(r#"{"rotated":{"subject":"next","roles":{"jobs":"admin"}}}"#)
        .unwrap();
    assert!(verifier.authenticate(&before).is_err());

    let mut after = HeaderMap::new();
    after.insert(header::AUTHORIZATION, "Bearer rotated".parse().unwrap());
    let principal = verifier.authenticate(&after).unwrap();
    assert_eq!(principal.subject(), Some("next"));
    assert!(principal.ensure("jobs", Role::Admin).is_ok());
}

#[test]
fn malformed_rotation_keeps_the_last_known_good_registry() {
    let verifier = required_auth().verifier();
    assert!(verifier.reload_json("not-json").is_err());

    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        "Bearer reader-token".parse().unwrap(),
    );
    let principal = verifier.authenticate(&headers).unwrap();
    assert!(principal.ensure("jobs", Role::Read).is_ok());
    assert!(principal.ensure("jobs", Role::Write).is_err());
}
// HANDWRITE-END
