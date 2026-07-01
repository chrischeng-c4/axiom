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

# Standardized projects/lumen/src/auth.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/auth.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AuthConfig` | projects/lumen/src/auth.rs | struct | pub | 69 |  |
| `AuthContext` | projects/lumen/src/auth.rs | enum | pub | 140 |  |
| `AuthErr` | projects/lumen/src/auth.rs | enum | pub | 193 |  |
| `LumenVerifier` | projects/lumen/src/auth.rs | struct | pub | 104 |  |
| `Role` | projects/lumen/src/auth.rs | enum | pub | 45 |  |
| `TokenClaims` | projects/lumen/src/auth.rs | struct | pub | 60 |  |
| `auth_middleware` | projects/lumen/src/auth.rs | function | pub | 183 | auth_middleware(     State(verifier): State<Arc<LumenVerifier>>,     req: Request,     next: Next, ) -> Response |
| `covers` | projects/lumen/src/auth.rs | function | pub | 53 | covers(self, needed: Role) -> bool |
| `ensure` | projects/lumen/src/auth.rs | function | pub | 149 | ensure(&self, collection_id: &str, needed: Role) -> Result<(), AuthErr> |
| `from_env` | projects/lumen/src/auth.rs | function | pub | 83 | from_env() -> Result<Self> |
| `new` | projects/lumen/src/auth.rs | function | pub | 110 | new(cfg: Arc<AuthConfig>) -> Self |
| `open` | projects/lumen/src/auth.rs | function | pub | 76 | open() -> Self |
| `subject` | projects/lumen/src/auth.rs | function | pub | 174 | subject(&self) -> Option<&str> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Bearer-token auth + per-collection RBAC.
//!
//! v1 keeps the verifier behind a trait so the static config-driven
//! implementation here can be swapped out later (k8s ServiceAccount JWT,
//! OIDC, mTLS-derived identity) without touching the handlers.
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
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use service_auth::{bearer_token, AuthError as ServiceAuthError, Verifier};

use crate::types::ApiError;

const WILDCARD_COLLECTION: &str = "*";
const TOKEN_REGISTRY_FILE_ENV: &str = "LUMEN_TOKEN_REGISTRY_FILE";
const LEGACY_TOKENS_ENV: &str = "LUMEN_TOKENS";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
pub enum Role {
    Read,
    Write,
    Admin,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
impl Role {
    pub fn covers(self, needed: Role) -> bool {
        self >= needed
    }
}

#[derive(Debug, Clone, Deserialize)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
pub struct TokenClaims {
    pub subject: String,
    /// `collection_id` → `Role`. The literal key `*` is a wildcard.
    #[serde(default)]
    pub roles: HashMap<String, Role>,
}

#[derive(Debug, Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
pub struct AuthConfig {
    pub required: bool,
    pub tokens: HashMap<String, TokenClaims>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
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
                other => bail!(
                    "LUMEN_AUTH must be `off`, `disabled`, or `required`; got `{other}`"
                ),
            },
            Err(std::env::VarError::NotPresent) => false,
            Err(e) => bail!("LUMEN_AUTH must be valid UTF-8: {e}"),
        };
        let tokens = match std::env::var(TOKEN_REGISTRY_FILE_ENV) {
            Ok(path) if !path.trim().is_empty() => {
                let json = std::fs::read_to_string(path.trim())
                    .with_context(|| format!("read {TOKEN_REGISTRY_FILE_ENV} `{}`", path.trim()))?;
                serde_json::from_str(&json)
                    .with_context(|| format!("{TOKEN_REGISTRY_FILE_ENV} must contain JSON"))?
            }
            _ => match std::env::var(LEGACY_TOKENS_ENV) {
                Ok(json) if !json.trim().is_empty() => serde_json::from_str(&json)
                    .with_context(|| format!("{LEGACY_TOKENS_ENV} must be JSON"))?,
                _ => HashMap::new(),
            },
        };
        if required && tokens.is_empty() {
            bail!(
                "LUMEN_AUTH=required requires a non-empty {TOKEN_REGISTRY_FILE_ENV} or {LEGACY_TOKENS_ENV}"
            );
        }
        Ok(Self { required, tokens })
    }

    fn lookup(&self, token: &str) -> Option<&TokenClaims> {
        self.tokens.get(token)
    }
}

/// Lumen's concrete verifier for the shared `service-auth` middleware.
#[derive(Debug, Clone)]
/// @spec projects/lumen/tech-design/logic/lumen-service-auth-convergence-delegate-middleware-to-shared-ver.md#logic
pub struct LumenVerifier {
    cfg: Arc<AuthConfig>,
}

/// @spec projects/lumen/tech-design/logic/lumen-service-auth-convergence-delegate-middleware-to-shared-ver.md#logic
impl LumenVerifier {
    pub fn new(cfg: Arc<AuthConfig>) -> Self {
        Self { cfg }
    }
}

/// @spec projects/lumen/tech-design/logic/lumen-service-auth-convergence-delegate-middleware-to-shared-ver.md#logic
impl Verifier for LumenVerifier {
    type Principal = AuthContext;

    fn authenticate(&self, headers: &HeaderMap) -> Result<AuthContext, ServiceAuthError> {
        match (self.cfg.required, bearer_token(headers)) {
            (false, None) => Ok(AuthContext::Open),
            (_, Some(t)) => self
                .cfg
                .lookup(t)
                .cloned()
                .map(AuthContext::Token)
                .ok_or(ServiceAuthError::Unauthenticated),
            (true, None) => Err(ServiceAuthError::Unauthenticated),
        }
    }

    fn required(&self) -> bool {
        self.cfg.required
    }
}

/// Resolved auth state attached to every request as an axum extension.
#[derive(Debug, Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
pub enum AuthContext {
    /// `LUMEN_AUTH=off` and no token was presented. Treated as full
    /// admin in development; production should set `required=true`.
    Open,
    Token(TokenClaims),
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
impl AuthContext {
    pub fn ensure(&self, collection_id: &str, needed: Role) -> Result<(), AuthErr> {
        match self {
            AuthContext::Open => Ok(()),
            AuthContext::Token(claims) => {
                let have = claims
                    .roles
                    .get(collection_id)
                    .or_else(|| claims.roles.get(WILDCARD_COLLECTION));
                match have {
                    Some(r) if r.covers(needed) => Ok(()),
                    Some(_) => Err(AuthErr::Forbidden {
                        subject: claims.subject.clone(),
                        needed,
                        collection_id: collection_id.to_string(),
                    }),
                    None => Err(AuthErr::Forbidden {
                        subject: claims.subject.clone(),
                        needed,
                        collection_id: collection_id.to_string(),
                    }),
                }
            }
        }
    }

    pub fn subject(&self) -> Option<&str> {
        match self {
            AuthContext::Open => None,
            AuthContext::Token(c) => Some(c.subject.as_str()),
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
pub async fn auth_middleware(
    State(verifier): State<Arc<LumenVerifier>>,
    req: Request,
    next: Next,
) -> Response {
    service_auth::auth_middleware::<LumenVerifier>(State(verifier), req, next).await
}

#[derive(Debug)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
pub enum AuthErr {
    Unauthenticated,
    Forbidden {
        subject: String,
        needed: Role,
        collection_id: String,
    },
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-auth-rs.md#source
impl IntoResponse for AuthErr {
    fn into_response(self) -> Response {
        match self {
            AuthErr::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    error: "unauthenticated".into(),
                    message: "valid bearer token required".into(),
                }),
            )
                .into_response(),
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

    #[test]
    fn role_covers() {
        assert!(Role::Admin.covers(Role::Read));
        assert!(Role::Admin.covers(Role::Admin));
        assert!(!Role::Read.covers(Role::Admin));
        assert!(Role::Write.covers(Role::Read));
    }

    #[test]
    fn open_context_allows_everything() {
        assert!(AuthContext::Open.ensure("any", Role::Admin).is_ok());
    }

    #[test]
    fn per_collection_role_enforced() {
        let ctx = AuthContext::Token(token(&[("users", Role::Read)]));
        assert!(ctx.ensure("users", Role::Read).is_ok());
        assert!(ctx.ensure("users", Role::Write).is_err());
        assert!(ctx.ensure("other", Role::Read).is_err());
    }

    #[test]
    fn wildcard_collection_covers_all() {
        let ctx = AuthContext::Token(token(&[("*", Role::Write)]));
        assert!(ctx.ensure("any", Role::Read).is_ok());
        assert!(ctx.ensure("any", Role::Write).is_ok());
        assert!(ctx.ensure("any", Role::Admin).is_err());
    }

    #[test]
    fn specific_collection_role_overrides_no_wildcard() {
        // Per-collection role without wildcard — only that collection.
        let ctx = AuthContext::Token(token(&[("users", Role::Admin)]));
        assert!(ctx.ensure("users", Role::Admin).is_ok());
        assert!(ctx.ensure("other", Role::Read).is_err());
    }

    #[test]
    fn subject_helper_returns_some_for_token_and_none_for_open() {
        let ctx = AuthContext::Token(token(&[("u", Role::Read)]));
        assert_eq!(ctx.subject(), Some("tester"));
        assert_eq!(AuthContext::Open.subject(), None);
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
    fn role_compare_total_order() {
        let mut roles = vec![Role::Admin, Role::Read, Role::Write];
        roles.sort();
        assert_eq!(roles, vec![Role::Read, Role::Write, Role::Admin]);
    }

    #[test]
    fn auth_config_open_has_no_tokens() {
        let cfg = AuthConfig::open();
        assert!(!cfg.required);
        assert!(cfg.tokens.is_empty());
        assert!(cfg.lookup("anything").is_none());
    }

    #[test]
    fn auth_config_lookup_returns_claims() {
        let cfg = AuthConfig {
            required: true,
            tokens: HashMap::from([("abc".to_string(), token(&[("u", Role::Write)]))]),
        };
        assert!(cfg.lookup("abc").is_some());
        assert!(cfg.lookup("xyz").is_none());
    }

    #[test]
    fn lumen_verifier_open_mode_without_bearer_returns_open_context() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::open()));
        let ctx = verifier.authenticate(&HeaderMap::new()).unwrap();
        assert!(matches!(ctx, AuthContext::Open));
    }

    #[test]
    fn lumen_verifier_known_bearer_returns_token_context() {
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
    }

    #[test]
    fn lumen_verifier_invalid_bearer_rejects_with_shared_unauthenticated() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig {
            required: true,
            tokens: HashMap::from([("abc".to_string(), token(&[("u", Role::Read)]))]),
        }));
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer nope".parse().unwrap(),
        );
        let err = verifier.authenticate(&headers).unwrap_err();
        assert!(matches!(err, ServiceAuthError::Unauthenticated));
    }

    #[test]
    fn lumen_verifier_required_missing_bearer_rejects_with_shared_unauthenticated() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig {
            required: true,
            tokens: HashMap::new(),
        }));
        let err = verifier.authenticate(&HeaderMap::new()).unwrap_err();
        assert!(matches!(err, ServiceAuthError::Unauthenticated));
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
        assert_eq!(cfg.lookup("file-token").unwrap().subject, "alice");
        assert!(cfg.lookup("env-token").is_none());
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
        assert_eq!(cfg.lookup("t1").unwrap().subject, "alice");
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

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/auth.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/auth.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
