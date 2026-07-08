// SPEC-MANAGED: libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Static bearer-token role-map RBAC — the reusable model behind the
//! archetype's `<SVC>_AUTH=off|required` + `<SVC>_TOKEN_REGISTRY_FILE`
//! contract (originally lumen's hand-rolled `src/auth.rs`, generalized here
//! so keep/loom/relay/beam don't each fork it).
//!
//! ## Shape
//!
//! - [`Role`]: a hierarchy, `Admin` ⊇ `Write` ⊇ `Read`, compared with
//!   [`Role::covers`].
//! - [`TokenClaims`]: a bearer token's `subject` plus its `roles`, keyed by a
//!   generic **resource** string (lumen's `collection_id`, keep's
//!   `namespace`, ...). The literal key `*` is a wildcard grant applied when
//!   no more specific entry matches.
//! - [`load_registry`]: parse the token→claims map from a registry-file path
//!   (production, mounted from a Secret) or legacy inline JSON, failing fast
//!   when auth is required but the resolved registry ends up empty. Env-var
//!   *naming* stays the caller's concern — this fn only knows the resolved
//!   values plus the label strings to use in error context, so the wording
//!   stays byte-identical to whatever env vars a service actually reads.
//! - [`StaticRoleMapVerifier`]: a [`Verifier`] over that registry —
//!   `authenticate` resolves a bearer token to a [`RoleMapPrincipal`] (or
//!   `Open`, in non-required/dev mode, when no token is presented).
//! - [`RoleMapPrincipal::ensure`]: the per-resource authorization check a
//!   handler runs after authentication — rejects (as a structured
//!   [`RoleMapDenied`]) unless the principal's claim on the resource (or its
//!   wildcard grant) covers the needed role.

use std::collections::HashMap;

use anyhow::{bail, Context as _, Result};
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::error::AuthError;
use crate::middleware::bearer_token;
use crate::verifier::Verifier;

const WILDCARD_RESOURCE: &str = "*";

/// Role hierarchy: `Admin` ⊇ `Write` ⊇ `Read`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
pub enum Role {
    Read,
    Write,
    Admin,
}

/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
impl Role {
    /// Whether this role meets or exceeds `needed`.
    pub fn covers(self, needed: Role) -> bool {
        self >= needed
    }
}

/// A bearer token's resolved claims: who (`subject`) and what they may do,
/// keyed by a generic resource string. `*` is a wildcard grant applied when
/// no more specific entry matches.
#[derive(Debug, Clone, Deserialize)]
/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
pub struct TokenClaims {
    pub subject: String,
    /// `resource` → `Role`. The literal key `*` is a wildcard.
    #[serde(default)]
    pub roles: HashMap<String, Role>,
}

/// Load a token registry: a registry-file path (preferred, production
/// shape) or legacy inline JSON, in that priority order — same
/// `{ "<token>": { "subject": "...", "roles": { "<resource>|*": "read|write|admin" } } }`
/// shape either way. Fails fast when `required` is true but the resolved
/// registry ends up empty (a server configured to mandate auth but unable to
/// ever authenticate anyone is a startup misconfiguration, not a
/// per-request 401). `registry_file_env`/`legacy_tokens_env` only word the
/// error context; naming the actual env vars stays the caller's job so the
/// message matches whatever the service calls them.
/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
pub fn load_registry(
    required: bool,
    registry_file_env: &str,
    registry_file: Option<&str>,
    legacy_tokens_env: &str,
    legacy_tokens_json: Option<&str>,
) -> Result<HashMap<String, TokenClaims>> {
    let tokens = match registry_file {
        Some(path) if !path.trim().is_empty() => {
            let json = std::fs::read_to_string(path.trim())
                .with_context(|| format!("read {registry_file_env} `{}`", path.trim()))?;
            serde_json::from_str(&json)
                .with_context(|| format!("{registry_file_env} must contain JSON"))?
        }
        _ => match legacy_tokens_json {
            Some(json) if !json.trim().is_empty() => serde_json::from_str(json)
                .with_context(|| format!("{legacy_tokens_env} must be JSON"))?,
            _ => HashMap::new(),
        },
    };
    if required && tokens.is_empty() {
        bail!(
            "auth required but no tokens: set a non-empty {registry_file_env} or {legacy_tokens_env}"
        );
    }
    Ok(tokens)
}

/// The resolved principal for a request: `Open` (auth disabled, no bearer
/// presented) or an authenticated token's claims.
#[derive(Debug, Clone)]
/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
pub enum RoleMapPrincipal {
    /// Auth is disabled and no bearer was presented. Treated as full admin
    /// in development; production should run with auth required.
    Open,
    Token(TokenClaims),
}

/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
impl RoleMapPrincipal {
    /// The per-resource authorization check a handler runs after
    /// authentication: `Open` always passes (dev mode); a token must carry a
    /// claim on `resource` (or the wildcard `*`) that covers `needed`.
    pub fn ensure(&self, resource: &str, needed: Role) -> Result<(), RoleMapDenied> {
        match self {
            RoleMapPrincipal::Open => Ok(()),
            RoleMapPrincipal::Token(claims) => {
                let have = claims
                    .roles
                    .get(resource)
                    .or_else(|| claims.roles.get(WILDCARD_RESOURCE));
                match have {
                    Some(r) if r.covers(needed) => Ok(()),
                    _ => Err(RoleMapDenied {
                        subject: claims.subject.clone(),
                        needed,
                        resource: resource.to_string(),
                    }),
                }
            }
        }
    }

    pub fn subject(&self) -> Option<&str> {
        match self {
            RoleMapPrincipal::Open => None,
            RoleMapPrincipal::Token(c) => Some(c.subject.as_str()),
        }
    }
}

/// Why [`RoleMapPrincipal::ensure`] rejected a request: a valid principal
/// lacking `needed` on `resource`. Structured (not pre-rendered) so a
/// service can build its own message / audit log the way lumen does.
#[derive(Debug, Clone)]
/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
pub struct RoleMapDenied {
    pub subject: String,
    pub needed: Role,
    pub resource: String,
}

/// A [`Verifier`] over a static, config-driven token→claims registry — the
/// archetype's `<SVC>_AUTH=off|required` shape.
#[derive(Debug, Clone)]
/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
pub struct StaticRoleMapVerifier {
    required: bool,
    tokens: HashMap<String, TokenClaims>,
}

/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
impl StaticRoleMapVerifier {
    pub fn new(required: bool, tokens: HashMap<String, TokenClaims>) -> Self {
        Self { required, tokens }
    }

    /// Open/dev verifier: auth disabled, no tokens.
    pub fn open() -> Self {
        Self::new(false, HashMap::new())
    }

    fn lookup(&self, token: &str) -> Option<&TokenClaims> {
        self.tokens.get(token)
    }
}

/// @spec libs/service-auth/tech-design/semantic/source/libs-service-auth-src-role-map-rs.md#source
impl Verifier for StaticRoleMapVerifier {
    type Principal = RoleMapPrincipal;

    fn authenticate(&self, headers: &HeaderMap) -> Result<RoleMapPrincipal, AuthError> {
        match (self.required, bearer_token(headers)) {
            (false, None) => Ok(RoleMapPrincipal::Open),
            (_, Some(t)) => self
                .lookup(t)
                .cloned()
                .map(RoleMapPrincipal::Token)
                .ok_or(AuthError::Unauthenticated),
            (true, None) => Err(AuthError::Unauthenticated),
        }
    }

    fn required(&self) -> bool {
        self.required
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(roles: &[(&str, Role)]) -> TokenClaims {
        TokenClaims {
            subject: "tester".into(),
            roles: roles
                .iter()
                .map(|(r, role)| (r.to_string(), *role))
                .collect(),
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
    fn role_compare_total_order() {
        let mut roles = vec![Role::Admin, Role::Read, Role::Write];
        roles.sort();
        assert_eq!(roles, vec![Role::Read, Role::Write, Role::Admin]);
    }

    #[test]
    fn open_principal_allows_everything() {
        assert!(RoleMapPrincipal::Open.ensure("any", Role::Admin).is_ok());
        assert_eq!(RoleMapPrincipal::Open.subject(), None);
    }

    #[test]
    fn per_resource_role_enforced() {
        let p = RoleMapPrincipal::Token(token(&[("users", Role::Read)]));
        assert!(p.ensure("users", Role::Read).is_ok());
        assert!(p.ensure("users", Role::Write).is_err());
        assert!(p.ensure("other", Role::Read).is_err());
    }

    #[test]
    fn wildcard_resource_covers_all() {
        let p = RoleMapPrincipal::Token(token(&[("*", Role::Write)]));
        assert!(p.ensure("any", Role::Read).is_ok());
        assert!(p.ensure("any", Role::Write).is_ok());
        assert!(p.ensure("any", Role::Admin).is_err());
    }

    #[test]
    fn specific_resource_role_overrides_no_wildcard() {
        // Per-resource role without a wildcard grant — only that resource.
        let p = RoleMapPrincipal::Token(token(&[("users", Role::Admin)]));
        assert!(p.ensure("users", Role::Admin).is_ok());
        assert!(p.ensure("other", Role::Read).is_err());
    }

    #[test]
    fn ensure_denied_carries_structured_fields() {
        let p = RoleMapPrincipal::Token(token(&[("users", Role::Read)]));
        let denied = p.ensure("users", Role::Admin).unwrap_err();
        assert_eq!(denied.subject, "tester");
        assert_eq!(denied.needed, Role::Admin);
        assert_eq!(denied.resource, "users");
    }

    #[test]
    fn principal_subject_returns_some_for_token_and_none_for_open() {
        let p = RoleMapPrincipal::Token(token(&[("u", Role::Read)]));
        assert_eq!(p.subject(), Some("tester"));
        assert_eq!(RoleMapPrincipal::Open.subject(), None);
    }

    #[test]
    fn static_role_map_verifier_open_mode_without_bearer_returns_open_principal() {
        let verifier = StaticRoleMapVerifier::open();
        let p = verifier.authenticate(&HeaderMap::new()).unwrap();
        assert!(matches!(p, RoleMapPrincipal::Open));
    }

    #[test]
    fn static_role_map_verifier_known_bearer_returns_token_principal() {
        let verifier = StaticRoleMapVerifier::new(
            true,
            HashMap::from([("abc".to_string(), token(&[("u", Role::Write)]))]),
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer abc".parse().unwrap(),
        );
        let p = verifier.authenticate(&headers).unwrap();
        assert_eq!(p.subject(), Some("tester"));
        assert!(p.ensure("u", Role::Write).is_ok());
    }

    #[test]
    fn static_role_map_verifier_invalid_bearer_rejects_unauthenticated() {
        let verifier = StaticRoleMapVerifier::new(
            true,
            HashMap::from([("abc".to_string(), token(&[("u", Role::Read)]))]),
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer nope".parse().unwrap(),
        );
        let err = verifier.authenticate(&headers).unwrap_err();
        assert!(matches!(err, AuthError::Unauthenticated));
    }

    #[test]
    fn static_role_map_verifier_required_missing_bearer_rejects_unauthenticated() {
        let verifier = StaticRoleMapVerifier::new(true, HashMap::new());
        let err = verifier.authenticate(&HeaderMap::new()).unwrap_err();
        assert!(matches!(err, AuthError::Unauthenticated));
    }

    #[test]
    fn load_registry_empty_when_neither_source_set() {
        let tokens = load_registry(false, "REGISTRY_FILE", None, "LEGACY_TOKENS", None).unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn load_registry_prefers_file_over_legacy_json() {
        let path = std::env::temp_dir().join(format!(
            "service-auth-role-map-test-{}.json",
            std::process::id()
        ));
        std::fs::write(
            &path,
            r#"{"file-token": {"subject": "alice", "roles": {"u": "write"}}}"#,
        )
        .unwrap();
        let tokens = load_registry(
            true,
            "REGISTRY_FILE",
            Some(path.to_str().unwrap()),
            "LEGACY_TOKENS",
            Some(r#"{"env-token": {"subject": "env", "roles": {"*": "admin"}}}"#),
        )
        .unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens.get("file-token").unwrap().subject, "alice");
        assert!(tokens.get("env-token").is_none());
    }

    #[test]
    fn load_registry_falls_back_to_legacy_json() {
        let tokens = load_registry(
            true,
            "REGISTRY_FILE",
            None,
            "LEGACY_TOKENS",
            Some(r#"{"t1": {"subject": "alice", "roles": {"u": "write"}}}"#),
        )
        .unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens.get("t1").unwrap().subject, "alice");
    }

    #[test]
    fn load_registry_required_without_tokens_fails_fast() {
        let err = load_registry(true, "REGISTRY_FILE", None, "LEGACY_TOKENS", None).unwrap_err();
        assert!(err.to_string().contains("REGISTRY_FILE"));
        assert!(err.to_string().contains("LEGACY_TOKENS"));
    }

    #[test]
    fn load_registry_rejects_bad_json() {
        let err = load_registry(
            false,
            "REGISTRY_FILE",
            None,
            "LEGACY_TOKENS",
            Some("not-json"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("LEGACY_TOKENS"));
    }
}
// CODEGEN-END
