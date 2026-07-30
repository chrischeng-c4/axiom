---
id: projects-lumen-src-auth-rs
capability_refs:
  - id: "security-hardening"
    role: primary
    claim: "bearer-token-auth-lumen-auth"
    coverage: partial
    rationale: "auth.rs owns Lumen bearer-token auth, shared service-auth verifier adoption, and per-collection RBAC enforcement."
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/auth.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/src/auth.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AuthConfig` | apps/lumen/src/auth.rs | struct | pub | 70 |  |
| `AuthContext` | apps/lumen/src/auth.rs | struct | pub | 226 |  |
| `AuthErr` | apps/lumen/src/auth.rs | enum | pub | 250 |  |
| `LumenAuthEventSink` | apps/lumen/src/auth.rs | struct | pub | 196 |  |
| `LumenVerifier` | apps/lumen/src/auth.rs | struct | pub | 129 |  |
| `auth_middleware` | apps/lumen/src/auth.rs | function | pub | 240 | auth_middleware(     State(verifier): State<Arc<LumenVerifier>>,     req: Request,     next: Next, ) -> Response |
| `ensure` | apps/lumen/src/auth.rs | function | pub | 230 | ensure(&self, collection_id: &str, needed: Role) -> Result<(), AuthErr> |
| `from_env` | apps/lumen/src/auth.rs | function | pub | 92 | from_env() -> Result<Self> |
| `new` | apps/lumen/src/auth.rs | function | pub | 133 | new(cfg: Arc<AuthConfig>) -> Self |
| `new` | apps/lumen/src/auth.rs | function | pub | 202 | new(engine: Arc<Engine>) -> Self |
| `open` | apps/lumen/src/auth.rs | function | pub | 77 | open() -> Self |
| `reload_file` | apps/lumen/src/auth.rs | function | pub | 166 | reload_file(&self, path: impl AsRef<Path>) -> Result<u64> |
| `reload_json` | apps/lumen/src/auth.rs | function | pub | 170 | reload_json(&self, json: &str) -> Result<u64> |
| `subject` | apps/lumen/src/auth.rs | function | pub | 234 | subject(&self) -> Option<&str> |
| `with_metrics` | apps/lumen/src/auth.rs | function | pub | 148 | with_metrics(cfg: Arc<AuthConfig>, engine: Arc<Engine>) -> Self |
| `with_tokens` | apps/lumen/src/auth.rs | function | pub | 85 | with_tokens(required: bool, tokens: HashMap<String, TokenClaims>) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

```rust
// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Bearer-token auth + per-collection RBAC.
//!
//! Thin lumen adapter over `service_auth::role_map`'s generic static
//! role-map model (`Role` / `TokenClaims` / `StaticRoleMapVerifier`): this
//! file only owns lumen's env wiring, `LumenVerifier`'s `Verifier` impl
//! (a newtype over the shared verifier), and the `AuthErr` → `ApiError`
//! mapping. The role hierarchy, the wildcard-`*` grant, and the
//! per-resource `ensure` check all live in `libs/service-auth`.
//!
//! ## Configuration
//!
//! Env (read by [`AuthConfig::from_env`]):
//!
//! - `LUMEN_AUTH=off|required` — default `off` (dev). `required` rejects
//!   requests without a bearer token.
//! - `LUMEN_TOKEN_REGISTRY_FILE` — the registry file, mounted from a
//!   Kubernetes Secret or a Secret Manager CSI projection. The operator sets
//!   it from `spec.tokensSecret` or `spec.tokensSecretProviderClass`; it is
//!   the *only* way to supply credentials, because a credential passed inline
//!   in the environment is a credential in `kubectl describe pod`. JSON, with
//!   two disjoint namespaces (#2678):
//!
//!   ```json
//!   { "tokens":     { "<secret>":  { "subject": "...", "roles": { "<collection_id>|*": "read|write|admin" } } },
//!     "identities": { "<email>":   { "subject": "...", "roles": { "<collection_id>|*": "read|write|admin" } } } }
//!   ```
//!
//!   `tokens` is keyed by the bearer secret; `identities` by a Google email an
//!   identity provider has verified. They never share a key namespace, so a
//!   bearer secret spelled like an email cannot reach an identity's grants.
//!   A flat `{ "<secret>": {...} }` document still parses as `tokens`. The
//!   wildcard collection `*` grants the role on every collection.
//!
//! ## Role precedence
//!
//! `admin` ⊇ `write` ⊇ `read`. A handler asks for the minimum role it
//! needs; [`AuthContext::ensure`] returns 403 unless the token's claim
//! on the target collection meets or exceeds that bar.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use service_auth::{
    AuditedRoleMapPrincipal, AuthError as ServiceAuthError, AuthEvent, AuthEventSink, Registry,
    ReloadableRoleMapVerifier, RoleMapDenied, TracingAuthEventSink, Verifier,
};

pub use service_auth::{Role, TokenClaims};

use crate::storage::Engine;
use crate::types::ApiError;

/// Environment variable naming the Secret/CSI-projected token registry.
/// The serving adapter watches this exact file after successful startup so a
/// validated replacement updates the live request verifier without a restart.
pub const TOKEN_REGISTRY_FILE_ENV: &str = "LUMEN_TOKEN_REGISTRY_FILE";

#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
pub struct AuthConfig {
    pub required: bool,
    pub registry: Registry,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
impl AuthConfig {
    pub fn open() -> Self {
        Self {
            required: false,
            registry: Registry::default(),
        }
    }

    /// A bearer-only config, the shape most tests want.
    pub fn with_tokens(required: bool, tokens: HashMap<String, TokenClaims>) -> Self {
        Self {
            required,
            registry: Registry::from_tokens(tokens),
        }
    }

    pub fn from_env() -> Result<Self> {
        let required = match std::env::var("LUMEN_AUTH") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "required" => true,
                "off" | "disabled" => false,
                other => {
                    bail!("LUMEN_AUTH must be `off`, `disabled`, or `required`; got `{other}`")
                }
            },
            Err(std::env::VarError::NotPresent) => false,
            Err(e) => bail!("LUMEN_AUTH must be valid UTF-8: {e}"),
        };
        // The env var is what this process reads, but it is not what the
        // operator set — naming only `LUMEN_TOKEN_REGISTRY_FILE` sends whoever
        // reads this line looking for an env var they cannot edit. The two CR
        // fields that produce it are named here so the message points at
        // something actionable (#2678, R5).
        let registry = service_auth::load_registry_file(
            required,
            TOKEN_REGISTRY_FILE_ENV,
            std::env::var(TOKEN_REGISTRY_FILE_ENV).ok().as_deref(),
        )
        .with_context(|| {
            format!(
                "{TOKEN_REGISTRY_FILE_ENV} comes from the Lumen resource's `spec.tokensSecret` \
                 (a Kubernetes Secret) or `spec.tokensSecretProviderClass` (a Secret Manager CSI \
                 projection); set exactly one of them when `spec.auth: required`"
            )
        })?;
        Ok(Self { required, registry })
    }
}

/// Lumen's concrete verifier for the shared `service-auth` middleware: a
/// thin newtype over `service_auth::ReloadableRoleMapVerifier`.
#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/logic/lumen-service-auth-convergence-delegate-middleware-to-shared-ver.md#logic
pub struct LumenVerifier(Arc<ReloadableRoleMapVerifier>);

/// @spec apps/lumen/tech-design/logic/lumen-service-auth-convergence-delegate-middleware-to-shared-ver.md#logic
impl LumenVerifier {
    pub fn new(cfg: Arc<AuthConfig>) -> Self {
        Self(Arc::new(ReloadableRoleMapVerifier::with_registry_and_sink(
            cfg.required,
            cfg.registry.clone(),
            Arc::new(TracingAuthEventSink),
        )))
    }

    /// #2475: same wiring as [`Self::new`], but the registry-reload sink
    /// additionally counts failed/successful hot-reloads onto `engine`'s
    /// `/metrics` surface (`crate::auth::LumenAuthEventSink`) so
    /// `render::prometheus_rule`'s `LumenAuthRegistryReloadFailing` alert
    /// has a real series to read. `AppState::with_components` is the only
    /// production caller; test call sites keep using `Self::new`, which
    /// has no metrics side effect.
    pub fn with_metrics(cfg: Arc<AuthConfig>, engine: Arc<Engine>) -> Self {
        Self(Arc::new(ReloadableRoleMapVerifier::with_registry_and_sink(
            cfg.required,
            cfg.registry.clone(),
            Arc::new(LumenAuthEventSink::new(engine)),
        )))
    }

    /// The shared reloadable verifier owned by this service wrapper. The
    /// serving adapter passes this same instance to the Secret/CSI file
    /// watcher, while the HTTP middleware keeps using the wrapper itself.
    pub fn registry_verifier(&self) -> Arc<ReloadableRoleMapVerifier> {
        Arc::clone(&self.0)
    }

    /// Explicit credential-rotation boundary. Parsing/validation completes
    /// before the shared verifier swaps its snapshot; errors preserve the
    /// last-known-good registry.
    pub fn reload_file(&self, path: impl AsRef<Path>) -> Result<u64> {
        self.0.reload_file(path)
    }

    pub fn reload_json(&self, json: &str) -> Result<u64> {
        self.0.reload_json(json)
    }
}

/// @spec apps/lumen/tech-design/logic/lumen-service-auth-convergence-delegate-middleware-to-shared-ver.md#logic
impl Verifier for LumenVerifier {
    type Principal = AuthContext;

    fn authenticate(&self, headers: &HeaderMap) -> Result<AuthContext, ServiceAuthError> {
        self.0.authenticate(headers).map(AuthContext)
    }

    fn required(&self) -> bool {
        self.0.required()
    }
}

/// #2475: registry-reload event sink used by `LumenVerifier::with_metrics`.
/// Delegates every event to [`TracingAuthEventSink`] for unchanged log
/// parity, and additionally records `RegistryReload` outcomes onto
/// `engine`'s `Metrics` — the same instance `GET /metrics` renders — so a
/// silently-stuck rotation (the verifier keeps serving the last known-good
/// registry on failure, by design) is externally observable.
#[derive(Debug)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
pub struct LumenAuthEventSink {
    engine: Arc<Engine>,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
impl LumenAuthEventSink {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
impl AuthEventSink for LumenAuthEventSink {
    fn record(&self, event: &AuthEvent) {
        TracingAuthEventSink.record(event);
        if let AuthEvent::RegistryReload { applied, .. } = event {
            if *applied {
                self.engine.metrics().touch_auth_registry_reload_success();
            } else {
                self.engine.metrics().incr_auth_registry_reload_failure();
            }
        }
    }
}

/// Resolved auth state attached to every request as an axum extension. A
/// thin newtype over the shared [`AuditedRoleMapPrincipal`] so [`ensure`](Self::ensure)
/// can map its rejection into lumen's own [`AuthErr`] / `ApiError` shape.
#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
pub struct AuthContext(AuditedRoleMapPrincipal);

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
impl AuthContext {
    pub fn ensure(&self, collection_id: &str, needed: Role) -> Result<(), AuthErr> {
        self.0.ensure(collection_id, needed).map_err(AuthErr::from)
    }

    pub fn subject(&self) -> Option<&str> {
        self.0.subject()
    }
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
pub async fn auth_middleware(
    State(verifier): State<Arc<LumenVerifier>>,
    req: Request,
    next: Next,
) -> Response {
    service_auth::auth_middleware::<LumenVerifier>(State(verifier), req, next).await
}

#[derive(Debug)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
pub enum AuthErr {
    Forbidden {
        subject: String,
        needed: Role,
        collection_id: String,
    },
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
impl From<RoleMapDenied> for AuthErr {
    fn from(denied: RoleMapDenied) -> Self {
        AuthErr::Forbidden {
            subject: denied.subject,
            needed: denied.needed,
            collection_id: denied.resource,
        }
    }
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
impl IntoResponse for AuthErr {
    fn into_response(self) -> Response {
        match self {
            AuthErr::Forbidden {
                subject,
                needed,
                collection_id,
            } => {
                tracing::warn!(
                    target: "lumen.audit",
                    event = "rbac_denied",
                    %subject,
                    collection_id = %collection_id,
                    needed = ?needed,
                );
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiError {
                        error: "forbidden".into(),
                        message: format!(
                            "subject `{subject}` lacks {needed:?} on `{collection_id}`"
                        ),
                    }),
                )
                    .into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(roles: &[(&str, Role)]) -> TokenClaims {
        TokenClaims {
            subject: "tester".into(),
            roles: roles.iter().map(|(c, r)| (c.to_string(), *r)).collect(),
        }
    }

    // Process-global env mutex shared across the env-mutating tests.
    use std::sync::Mutex;
    static AUTH_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_auth_env() {
        unsafe {
            std::env::remove_var("LUMEN_AUTH");
            std::env::remove_var(TOKEN_REGISTRY_FILE_ENV);
        }
    }

    #[test]
    fn auth_config_open_has_no_tokens() {
        let cfg = AuthConfig::open();
        assert!(!cfg.required);
        assert!(cfg.registry.is_empty());
        assert!(cfg.registry.tokens.get("anything").is_none());
    }

    #[test]
    fn auth_config_construction_holds_tokens_by_map_key() {
        let cfg = AuthConfig::with_tokens(
            true,
            HashMap::from([("abc".to_string(), token(&[("u", Role::Write)]))]),
        );
        assert!(cfg.registry.tokens.get("abc").is_some());
        assert!(cfg.registry.tokens.get("xyz").is_none());
    }

    #[test]
    fn lumen_verifier_delegates_to_shared_reloadable_role_map_verifier() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::with_tokens(
            true,
            HashMap::from([("abc".to_string(), token(&[("u", Role::Write)]))]),
        )));
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer abc".parse().unwrap(),
        );
        let ctx = verifier.authenticate(&headers).unwrap();
        assert_eq!(ctx.subject(), Some("tester"));
        assert!(ctx.ensure("u", Role::Write).is_ok());

        // Unknown bearer still rejects via the shared verifier's Unauthenticated.
        let mut bad = HeaderMap::new();
        bad.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer nope".parse().unwrap(),
        );
        assert!(matches!(
            verifier.authenticate(&bad).unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));

        verifier
            .reload_json(r#"{"rotated":{"subject":"next","roles":{"u":"admin"}}}"#)
            .unwrap();
        assert!(verifier.authenticate(&headers).is_err());
        let mut rotated = HeaderMap::new();
        rotated.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer rotated".parse().unwrap(),
        );
        let ctx = verifier.authenticate(&rotated).unwrap();
        assert_eq!(ctx.subject(), Some("next"));
        assert!(ctx.ensure("u", Role::Admin).is_ok());
    }

    #[test]
    fn registry_handle_reloads_the_live_lumen_verifier() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::with_tokens(
            true,
            HashMap::from([("old".to_string(), token(&[("u", Role::Read)]))]),
        )));
        verifier
            .registry_verifier()
            .reload_json(r#"{"rotated":{"subject":"next","roles":{"u":"admin"}}}"#)
            .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer rotated".parse().unwrap(),
        );
        let context = verifier.authenticate(&headers).unwrap();
        assert_eq!(context.subject(), Some("next"));
        assert!(context.ensure("u", Role::Admin).is_ok());
    }

    #[test]
    fn auth_context_ensure_forbidden_maps_resource_to_collection_id() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::with_tokens(
            true,
            HashMap::from([("abc".to_string(), token(&[("users", Role::Read)]))]),
        )));
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer abc".parse().unwrap(),
        );
        let ctx = verifier.authenticate(&headers).unwrap();
        match ctx.ensure("users", Role::Write) {
            Err(AuthErr::Forbidden {
                subject,
                needed,
                collection_id,
            }) => {
                assert_eq!(subject, "tester");
                assert_eq!(needed, Role::Write);
                assert_eq!(collection_id, "users");
            }
            other => panic!("expected Forbidden, got {other:?}"),
        }
    }

    #[test]
    fn auth_context_open_allows_everything() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::open()));
        let ctx = verifier.authenticate(&HeaderMap::new()).unwrap();
        assert!(ctx.ensure("any", Role::Admin).is_ok());
        assert_eq!(ctx.subject(), None);
    }

    #[test]
    fn auth_config_from_env_open_when_unset() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        let cfg = AuthConfig::from_env().unwrap();
        assert!(!cfg.required);
        assert!(cfg.registry.is_empty());
    }

    #[test]
    fn auth_config_from_env_with_registry_file() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("token-registry.json");
        std::fs::write(
            &path,
            r#"{"file-token": {"subject": "alice", "roles": {"u": "write"}}}"#,
        )
        .unwrap();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
            std::env::set_var(TOKEN_REGISTRY_FILE_ENV, &path);
        }
        let cfg = AuthConfig::from_env().unwrap();
        assert!(cfg.required);
        assert_eq!(cfg.registry.len(), 1);
        assert_eq!(
            cfg.registry.tokens.get("file-token").unwrap().subject,
            "alice"
        );
        clear_auth_env();
    }

    /// #2678: the registry file is the only credential source, and it carries
    /// both namespaces. Identity entries survive the load rather than being
    /// dropped as unresolvable, because lumen has a Google verifier for them.
    #[test]
    fn auth_config_from_env_loads_both_namespaces() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("token-registry.json");
        std::fs::write(
            &path,
            r#"{"tokens":{"s3cret":{"subject":"svc","roles":{"u":"write"}}},
                "identities":{"dev@example.com":{"subject":"dev","roles":{"u":"read"}}}}"#,
        )
        .unwrap();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
            std::env::set_var(TOKEN_REGISTRY_FILE_ENV, &path);
        }
        let cfg = AuthConfig::from_env().unwrap();
        assert_eq!(cfg.registry.tokens.len(), 1);
        assert_eq!(cfg.registry.identities.len(), 1);
        // The namespaces stay disjoint all the way through lumen's loader.
        assert!(cfg.registry.tokens.get("dev@example.com").is_none());
        clear_auth_env();
    }

    /// R5: `spec.auth: required` with neither token source set must fail
    /// startup naming the *CR fields* an operator can act on, not only the
    /// env var the operator never sets by hand.
    #[test]
    fn auth_config_required_without_tokens_fails_fast_naming_the_cr_fields() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
        }
        let err = AuthConfig::from_env().unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains(TOKEN_REGISTRY_FILE_ENV), "{message}");
        assert!(message.contains("spec.tokensSecret"), "{message}");
        assert!(
            message.contains("spec.tokensSecretProviderClass"),
            "{message}"
        );
        clear_auth_env();
    }

    #[test]
    fn auth_config_from_env_rejects_unknown_auth_mode() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "require");
        }
        let err = AuthConfig::from_env().unwrap_err();
        assert!(err.to_string().contains("LUMEN_AUTH"));
        clear_auth_env();
    }

    /// #2475: `LumenVerifier::with_metrics`'s sink must count a failed
    /// hot-reload and stamp a successful one onto the `Engine`'s
    /// `/metrics` surface — the series `LumenAuthRegistryReloadFailing`
    /// reads.
    #[test]
    fn with_metrics_records_reload_failures_and_successes_on_engine() {
        let engine = Arc::new(crate::storage::Engine::new());
        let verifier = LumenVerifier::with_metrics(
            Arc::new(AuthConfig::with_tokens(
                true,
                HashMap::from([("abc".to_string(), token(&[("u", Role::Write)]))]),
            )),
            engine.clone(),
        );
        assert_eq!(
            engine.metrics().auth_registry_reload_failures_total.get(),
            0
        );

        assert!(verifier.reload_json("not-json").is_err());
        assert_eq!(
            engine.metrics().auth_registry_reload_failures_total.get(),
            1
        );

        verifier
            .reload_json(r#"{"rotated":{"subject":"next","roles":{"u":"admin"}}}"#)
            .unwrap();
        assert_eq!(
            engine.metrics().auth_registry_reload_failures_total.get(),
            1
        );
        assert!(engine.metrics().auth_registry_reload_success_unixtime.get() > 0);
    }

    #[test]
    fn auth_config_from_env_rejects_bad_json() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("token-registry.json");
        std::fs::write(&path, "not-json").unwrap();
        unsafe {
            std::env::set_var(TOKEN_REGISTRY_FILE_ENV, &path);
        }
        let err = AuthConfig::from_env().unwrap_err();
        assert!(err.to_string().contains(TOKEN_REGISTRY_FILE_ENV));
        clear_auth_env();
    }
}
// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/auth.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `apps/lumen/src/auth.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
