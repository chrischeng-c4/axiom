//! What the serving process must make of the two files and the audience list
//! the operator now renders for it (#2764, slice E).
//!
//! #2788 wired the *manifest* side: an instance-owned `ConfigMap` carrying
//! `identities.json`, a mount at a path named by `LUMEN_IDENTITY_REGISTRY_FILE`,
//! and `LUMEN_AUTH_GOOGLE_AUDIENCES` rendered as `identityAudiences.join(",")`.
//! Nothing reads either one. Until something does, the audience list is carried
//! but not enforced, which is the failure mode worth naming: an ID-token
//! verifier with no audience accepts every token Google mints, for anyone's
//! service, silently.
//!
//! These assertions are the contract for that read. They are deliberately
//! confined to `AuthConfig::from_env` — the env→config boundary — because that
//! is where the operator's rendering meets the process, and it is checkable
//! without a network, a JWKS source or a clock. Selecting the verifier and
//! swapping the middleware to the async one is the slice after this.
//!
//! Process env is global. Every test here serializes on `ENV_LOCK` and clears
//! all four variables on entry, so a variable set by one test can never leak
//! into another's `from_env`.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use lumen::auth::{
    AuthConfig, GOOGLE_AUDIENCES_ENV, IDENTITY_REGISTRY_FILE_ENV, TOKEN_REGISTRY_FILE_ENV,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// A bearer Secret, as `spec.tokensSecret` projects it.
const TOKENS_JSON: &str =
    r#"{"tokens":{"s3cr3t":{"subject":"tenant-bearer","roles":{"*":"read"}}}}"#;

/// An identity ConfigMap, as the operator renders `spec.identities`.
const IDENTITIES_JSON: &str =
    r#"{"identities":{"ops@acme.example":{"subject":"tenant-ops","roles":{"*":"admin"}}}}"#;

/// Clears every variable `from_env` reads, then holds the lock for the test.
fn env_guard() -> MutexGuard<'static, ()> {
    let guard = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe {
        std::env::remove_var("LUMEN_AUTH");
        std::env::remove_var(TOKEN_REGISTRY_FILE_ENV);
        std::env::remove_var(IDENTITY_REGISTRY_FILE_ENV);
        std::env::remove_var(GOOGLE_AUDIENCES_ENV);
    }
    guard
}

fn set(key: &str, value: &str) {
    unsafe { std::env::set_var(key, value) }
}

fn write_fixture(dir: &Path, name: &str, body: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write registry fixture");
    path.to_string_lossy().into_owned()
}

#[test]
fn both_registry_files_load_into_one_registry() {
    let _guard = env_guard();
    let dir = tempfile::tempdir().expect("tempdir");
    let tokens = write_fixture(dir.path(), "tokens.json", TOKENS_JSON);
    let identities = write_fixture(dir.path(), "identities.json", IDENTITIES_JSON);

    set("LUMEN_AUTH", "required");
    set(TOKEN_REGISTRY_FILE_ENV, &tokens);
    set(IDENTITY_REGISTRY_FILE_ENV, &identities);
    set(GOOGLE_AUDIENCES_ENV, "https://lumen.acme.internal");

    let cfg = AuthConfig::from_env().expect("both registry files load");

    assert!(cfg.required);
    assert_eq!(
        cfg.registry.tokens.len(),
        1,
        "the bearer Secret must survive the union"
    );
    assert_eq!(
        cfg.registry.identities.len(),
        1,
        "the identity ConfigMap must survive the union"
    );
    assert_eq!(
        cfg.registry.identities["ops@acme.example"].subject,
        "tenant-ops"
    );
}

#[test]
fn the_operators_comma_joined_audience_list_round_trips() {
    // The operator renders `spec.identityAudiences.join(",")`. Nothing has ever
    // asserted the separator at either end — the render test carries a single
    // audience and checks `contains`, which two audiences pass whether they are
    // split or not. A list that stays joined becomes one audience literally
    // spelled `a,b`, matching no token Google ever mints, and it fails as 401s
    // at request time rather than as an error at startup.
    let _guard = env_guard();
    let dir = tempfile::tempdir().expect("tempdir");
    let identities = write_fixture(dir.path(), "identities.json", IDENTITIES_JSON);

    set("LUMEN_AUTH", "required");
    set(IDENTITY_REGISTRY_FILE_ENV, &identities);
    set(GOOGLE_AUDIENCES_ENV, "https://a.example,https://b.example");

    let cfg = AuthConfig::from_env().expect("two audiences load");

    assert_eq!(
        cfg.google_audiences,
        vec![
            "https://a.example".to_string(),
            "https://b.example".to_string()
        ],
        "the comma the operator joined on is the comma this side splits on"
    );
}

#[test]
fn blank_audience_entries_never_become_an_audience() {
    // A hand-edited CR leaves trailing commas and stray spaces. An empty string
    // reaching `Validation::set_audience` is an audience that matches nothing,
    // and a whitespace-padded one is worse: it looks right in `kubectl describe`
    // and matches nothing either.
    let _guard = env_guard();
    let dir = tempfile::tempdir().expect("tempdir");
    let identities = write_fixture(dir.path(), "identities.json", IDENTITIES_JSON);

    set("LUMEN_AUTH", "required");
    set(IDENTITY_REGISTRY_FILE_ENV, &identities);
    set(GOOGLE_AUDIENCES_ENV, " https://a.example , , https://b.example ");

    let cfg = AuthConfig::from_env().expect("padded audiences load");

    assert_eq!(
        cfg.google_audiences,
        vec![
            "https://a.example".to_string(),
            "https://b.example".to_string()
        ]
    );
}

#[test]
fn an_identity_registry_without_an_audience_is_rejected_at_startup() {
    // The whole reason the audience list is p1. Failing closed here is the
    // difference between a pod that will not start and a pod that accepts
    // every Google-minted ID token in existence.
    let _guard = env_guard();
    let dir = tempfile::tempdir().expect("tempdir");
    let identities = write_fixture(dir.path(), "identities.json", IDENTITIES_JSON);

    set("LUMEN_AUTH", "required");
    set(IDENTITY_REGISTRY_FILE_ENV, &identities);

    let err = AuthConfig::from_env()
        .expect_err("identity grants with no audience must not start the process");
    let message = format!("{err:#}");

    assert!(
        message.contains(GOOGLE_AUDIENCES_ENV),
        "the error must name the variable an operator can act on, got: {message}"
    );
}

#[test]
fn an_identities_only_deployment_starts_with_no_bearer_secret_at_all() {
    // The end state this chain is heading for: identity-based auth replaces the
    // pre-shared bearer secret, so `spec.tokensSecret` is absent and no Secret
    // is mounted. A loader that still demands a bearer file would make that
    // deployment unstartable.
    let _guard = env_guard();
    let dir = tempfile::tempdir().expect("tempdir");
    let identities = write_fixture(dir.path(), "identities.json", IDENTITIES_JSON);

    set("LUMEN_AUTH", "required");
    set(IDENTITY_REGISTRY_FILE_ENV, &identities);
    set(GOOGLE_AUDIENCES_ENV, "https://lumen.acme.internal");

    let cfg = AuthConfig::from_env().expect("an identities-only deployment starts");

    assert!(cfg.registry.tokens.is_empty());
    assert_eq!(cfg.registry.identities.len(), 1);
}

#[test]
fn a_bearer_only_deployment_loads_exactly_as_it_did_before() {
    // The regression guard. Every lumen running today has a tokens Secret, no
    // identity ConfigMap and no audiences, and must keep loading.
    let _guard = env_guard();
    let dir = tempfile::tempdir().expect("tempdir");
    let tokens = write_fixture(dir.path(), "tokens.json", TOKENS_JSON);

    set("LUMEN_AUTH", "required");
    set(TOKEN_REGISTRY_FILE_ENV, &tokens);

    let cfg = AuthConfig::from_env().expect("a bearer-only deployment still loads");

    assert_eq!(cfg.registry.tokens.len(), 1);
    assert!(cfg.registry.identities.is_empty());
    assert!(
        cfg.google_audiences.is_empty(),
        "no identities configured means no audience list to carry"
    );
}

#[test]
fn a_bearer_secret_spelled_like_an_email_cannot_reach_an_identitys_grants() {
    // The two namespaces are disjoint by construction in `service-auth`. This
    // asserts lumen's own two-file load preserves that: the same string as a
    // bearer secret and as a verified identity resolves to two different
    // subjects with two different roles, and neither overwrites the other.
    let _guard = env_guard();
    let dir = tempfile::tempdir().expect("tempdir");
    let tokens = write_fixture(
        dir.path(),
        "tokens.json",
        r#"{"tokens":{"ops@acme.example":{"subject":"impostor","roles":{"*":"admin"}}}}"#,
    );
    let identities = write_fixture(
        dir.path(),
        "identities.json",
        r#"{"identities":{"ops@acme.example":{"subject":"tenant-ops","roles":{"*":"read"}}}}"#,
    );

    set("LUMEN_AUTH", "required");
    set(TOKEN_REGISTRY_FILE_ENV, &tokens);
    set(IDENTITY_REGISTRY_FILE_ENV, &identities);
    set(GOOGLE_AUDIENCES_ENV, "https://lumen.acme.internal");

    let cfg = AuthConfig::from_env().expect("the two namespaces coexist");

    assert_eq!(cfg.registry.tokens["ops@acme.example"].subject, "impostor");
    assert_eq!(
        cfg.registry.identities["ops@acme.example"].subject,
        "tenant-ops"
    );
}

#[test]
fn the_registry_error_never_names_a_crd_field_that_no_longer_exists() {
    // `spec.tokensSecretProviderClass` went away with the CSI path (#2764), so
    // an error still directing an operator to set it sends them to a field
    // `kubectl explain` does not list and the API server rejects. The whole
    // point of that message was to name something actionable.
    let _guard = env_guard();

    set("LUMEN_AUTH", "required");
    set(
        TOKEN_REGISTRY_FILE_ENV,
        "/nonexistent/lumen-oracle/tokens.json",
    );

    let err = AuthConfig::from_env().expect_err("an unreadable registry file fails the load");
    let message = format!("{err:#}");

    assert!(
        !message.contains("tokensSecretProviderClass"),
        "the error names a CR field that was removed: {message}"
    );
    assert!(
        message.contains("spec.tokensSecret"),
        "the error must still name the field that does exist: {message}"
    );
}

#[test]
fn a_malformed_identity_file_fails_the_load_naming_that_file() {
    // Two sources means a partial registry is now a possible outcome, and a
    // partial registry serves 403s that look like a policy decision. The load
    // must fail, and it must say which of the two files was unreadable.
    let _guard = env_guard();
    let dir = tempfile::tempdir().expect("tempdir");
    let tokens = write_fixture(dir.path(), "tokens.json", TOKENS_JSON);
    let identities = write_fixture(dir.path(), "identities.json", "{ this is not json");

    set("LUMEN_AUTH", "required");
    set(TOKEN_REGISTRY_FILE_ENV, &tokens);
    set(IDENTITY_REGISTRY_FILE_ENV, &identities);
    set(GOOGLE_AUDIENCES_ENV, "https://lumen.acme.internal");

    let err = AuthConfig::from_env().expect_err("a malformed identity file fails the load");
    let message = format!("{err:#}");

    assert!(
        message.contains(IDENTITY_REGISTRY_FILE_ENV),
        "the error must name the identity file, not the bearer one, got: {message}"
    );
}
