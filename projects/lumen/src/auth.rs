// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-src.md#schema
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
//! - `LUMEN_TOKENS` — JSON: `{ "<token>": { "subject": "...", "roles":
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

use anyhow::{Context, Result};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};

use crate::types::ApiError;

const WILDCARD_COLLECTION: &str = "*";

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Read,
    Write,
    Admin,
}

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
impl Role {
    pub fn covers(self, needed: Role) -> bool {
        self >= needed
    }
}

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
#[derive(Debug, Clone, Deserialize)]
pub struct TokenClaims {
    pub subject: String,
    /// `collection_id` → `Role`. The literal key `*` is a wildcard.
    #[serde(default)]
    pub roles: HashMap<String, Role>,
}

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub required: bool,
    pub tokens: HashMap<String, TokenClaims>,
}

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
impl AuthConfig {
    pub fn open() -> Self {
        Self {
            required: false,
            tokens: HashMap::new(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let required = std::env::var("LUMEN_AUTH")
            .map(|v| v.eq_ignore_ascii_case("required"))
            .unwrap_or(false);
        let tokens = match std::env::var("LUMEN_TOKENS") {
            Ok(json) if !json.trim().is_empty() => {
                serde_json::from_str(&json).context("LUMEN_TOKENS must be JSON")?
            }
            _ => HashMap::new(),
        };
        Ok(Self { required, tokens })
    }

    fn lookup(&self, token: &str) -> Option<&TokenClaims> {
        self.tokens.get(token)
    }
}

/// Resolved auth state attached to every request as an axum extension.
/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
#[derive(Debug, Clone)]
pub enum AuthContext {
    /// `LUMEN_AUTH=off` and no token was presented. Treated as full
    /// admin in development; production should set `required=true`.
    Open,
    Token(TokenClaims),
}

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
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

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
pub async fn auth_middleware(
    State(cfg): State<Arc<AuthConfig>>,
    mut req: Request,
    next: Next,
) -> Response {
    if !cfg.required && cfg.tokens.is_empty() && !req.headers().contains_key(header::AUTHORIZATION)
    {
        req.extensions_mut().insert(AuthContext::Open);
        return next.run(req).await;
    }

    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_owned());

    let ctx = match (cfg.required, token.as_deref()) {
        (false, None) => AuthContext::Open,
        (_, Some(t)) => match cfg.lookup(t) {
            Some(claims) => AuthContext::Token(claims.clone()),
            None => return AuthErr::Unauthenticated.into_response(),
        },
        (true, None) => return AuthErr::Unauthenticated.into_response(),
    };
    req.extensions_mut().insert(ctx);
    next.run(req).await
}

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
#[derive(Debug)]
pub enum AuthErr {
    Unauthenticated,
    Forbidden {
        subject: String,
        needed: Role,
        collection_id: String,
    },
}

/// @spec projects/lumen/tech-design/semantic/lumen-src.md#schema
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

    // Process-global env mutex shared across the three env-mutating tests.
    use std::sync::Mutex;
    static AUTH_ENV_LOCK: Mutex<()> = Mutex::new(());

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
    fn auth_config_from_env_open_when_unset() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("LUMEN_AUTH");
            std::env::remove_var("LUMEN_TOKENS");
        }
        let cfg = AuthConfig::from_env().unwrap();
        assert!(!cfg.required);
        assert!(cfg.tokens.is_empty());
    }

    #[test]
    fn auth_config_from_env_with_tokens() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
            std::env::set_var(
                "LUMEN_TOKENS",
                r#"{"t1": {"subject": "alice", "roles": {"u": "write"}}}"#,
            );
        }
        let cfg = AuthConfig::from_env().unwrap();
        assert!(cfg.required);
        assert_eq!(cfg.tokens.len(), 1);
        assert_eq!(cfg.lookup("t1").unwrap().subject, "alice");
        unsafe {
            std::env::remove_var("LUMEN_AUTH");
            std::env::remove_var("LUMEN_TOKENS");
        }
    }

    #[test]
    fn auth_config_from_env_rejects_bad_json() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("LUMEN_TOKENS", "not-json");
        }
        let err = AuthConfig::from_env().unwrap_err();
        assert!(err.to_string().contains("LUMEN_TOKENS"));
        unsafe {
            std::env::remove_var("LUMEN_TOKENS");
        }
    }
}
// CODEGEN-END
