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
//! - `LUMEN_TOKEN_REGISTRY_FILE` — production registry file mounted from a
//!   Kubernetes Secret / Secret Manager projection. JSON: `{ "<token>":
//!   { "subject": "...", "roles": { "<collection_id>|*": "read|write|admin" } } }`.
//! - `LUMEN_TOKENS` — legacy inline JSON with the same shape:
//!   `{ "<token>": { "subject": "...", "roles":
//!   { "<collection_id>|*": "read|write|admin" } } }`. The wildcard
//!   collection `*` grants the role on every collection.
//!
//! ## Role precedence
//!
//! `admin` ⊇ `write` ⊇ `read`. A handler asks for the minimum role it
//! needs; [`AuthContext::ensure`] returns 403 unless the token's claim
//! on the target collection meets or exceeds that bar.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Result};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use service_auth::{
    AuditedRoleMapPrincipal, AuthError as ServiceAuthError, ReloadableRoleMapVerifier,
    RoleMapDenied, TracingAuthEventSink, Verifier,
};

pub use service_auth::{Role, TokenClaims};

use crate::types::ApiError;

/// Environment variable naming the Secret/CSI-projected token registry.
/// The serving adapter watches this exact file after successful startup so a
/// validated replacement updates the live request verifier without a restart.
pub const TOKEN_REGISTRY_FILE_ENV: &str = "LUMEN_TOKEN_REGISTRY_FILE";
const LEGACY_TOKENS_ENV: &str = "LUMEN_TOKENS";

#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
pub struct AuthConfig {
    pub required: bool,
    pub tokens: HashMap<String, TokenClaims>,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
impl AuthConfig {
    pub fn open() -> Self {
        Self {
            required: false,
            tokens: HashMap::new(),
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
        let tokens = service_auth::load_registry(
            required,
            TOKEN_REGISTRY_FILE_ENV,
            std::env::var(TOKEN_REGISTRY_FILE_ENV).ok().as_deref(),
            LEGACY_TOKENS_ENV,
            std::env::var(LEGACY_TOKENS_ENV).ok().as_deref(),
        )?;
        Ok(Self { required, tokens })
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
        Self(Arc::new(ReloadableRoleMapVerifier::with_sink(
            cfg.required,
            cfg.tokens.clone(),
            Arc::new(TracingAuthEventSink),
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
            std::env::remove_var(LEGACY_TOKENS_ENV);
        }
    }

    #[test]
    fn auth_config_open_has_no_tokens() {
        let cfg = AuthConfig::open();
        assert!(!cfg.required);
        assert!(cfg.tokens.is_empty());
        assert!(cfg.tokens.get("anything").is_none());
    }

    #[test]
    fn auth_config_construction_holds_tokens_by_map_key() {
        let cfg = AuthConfig {
            required: true,
            tokens: HashMap::from([("abc".to_string(), token(&[("u", Role::Write)]))]),
        };
        assert!(cfg.tokens.get("abc").is_some());
        assert!(cfg.tokens.get("xyz").is_none());
    }

    #[test]
    fn lumen_verifier_delegates_to_shared_reloadable_role_map_verifier() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig {
            required: true,
            tokens: HashMap::from([("abc".to_string(), token(&[("u", Role::Write)]))]),
        }));
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
        let verifier = LumenVerifier::new(Arc::new(AuthConfig {
            required: true,
            tokens: HashMap::from([("old".to_string(), token(&[("u", Role::Read)]))]),
        }));
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
        let verifier = LumenVerifier::new(Arc::new(AuthConfig {
            required: true,
            tokens: HashMap::from([("abc".to_string(), token(&[("users", Role::Read)]))]),
        }));
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
        assert!(cfg.tokens.is_empty());
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
            std::env::set_var(
                LEGACY_TOKENS_ENV,
                r#"{"env-token": {"subject": "env", "roles": {"*": "admin"}}}"#,
            );
        }
        let cfg = AuthConfig::from_env().unwrap();
        assert!(cfg.required);
        assert_eq!(cfg.tokens.len(), 1);
        assert_eq!(cfg.tokens.get("file-token").unwrap().subject, "alice");
        assert!(cfg.tokens.get("env-token").is_none());
        clear_auth_env();
    }

    #[test]
    fn auth_config_from_env_with_tokens() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
            std::env::set_var(
                LEGACY_TOKENS_ENV,
                r#"{"t1": {"subject": "alice", "roles": {"u": "write"}}}"#,
            );
        }
        let cfg = AuthConfig::from_env().unwrap();
        assert!(cfg.required);
        assert_eq!(cfg.tokens.len(), 1);
        assert_eq!(cfg.tokens.get("t1").unwrap().subject, "alice");
        clear_auth_env();
    }

    #[test]
    fn auth_config_required_without_tokens_fails_fast() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
        }
        let err = AuthConfig::from_env().unwrap_err();
        assert!(err.to_string().contains(TOKEN_REGISTRY_FILE_ENV));
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

    #[test]
    fn auth_config_from_env_rejects_bad_json() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var(LEGACY_TOKENS_ENV, "not-json");
        }
        let err = AuthConfig::from_env().unwrap_err();
        assert!(err.to_string().contains(LEGACY_TOKENS_ENV));
        clear_auth_env();
    }
}
// CODEGEN-END
