// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Request auth for the serving API — currently disabled-only.
//!
//! Lumen used to authenticate requests against a two-namespace registry file
//! (bearer secrets under `tokens`, provider-verified emails under
//! `identities`) that the operator projected from a Secret and a ConfigMap.
//! That model is retired: no token is issued to a caller at all, and
//! authorization moves to Kubernetes RBAC, so neither namespace has a caller
//! or an authority left. The registry is removed here (#2871) *before* its
//! replacement lands (#2869) precisely so it cannot survive as a fallback
//! that quietly keeps working.
//!
//! ## Configuration
//!
//! Env (read by [`AuthConfig::from_env`]):
//!
//! - `LUMEN_AUTH=off|disabled|required` — default `off`. `off`/`disabled`
//!   serve without authentication. `required` currently has no verifier
//!   behind it, so [`AuthConfig::from_env`] refuses to start: an open API is
//!   never an acceptable degradation of a request for a closed one.
//!
//! There is no credential file, no credential env var, and no credential
//! field on the CR. What the replacement will use instead — a short-lived,
//! audience-bound Kubernetes ServiceAccount token checked with TokenReview
//! and authorized with SubjectAccessReview — is phase 2's work.
//!
//! ## Role precedence
//!
//! `admin` ⊇ `write` ⊇ `read`. A handler asks for the minimum role it
//! needs; [`AuthContext::ensure`] returns 403 unless the request's principal
//! meets or exceeds that bar. With auth disabled every request resolves to
//! the shared verifier's `Open` principal, which passes every check — the
//! `ensure` call sites stay in place so phase 2 has somewhere to reconnect
//! a real authorization decision.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Result};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use service_auth::{
    AuthError as ServiceAuthError, RoleMapDenied, RoleMapPrincipal, StaticRoleMapVerifier, Verifier,
};

pub use service_auth::Role;

use crate::types::ApiError;

#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
pub struct AuthConfig {
    /// Always `false` for any config [`AuthConfig::from_env`] returns — it
    /// refuses to build a `required` one while no verifier exists. The field
    /// survives because it is what a caller constructing state by hand still
    /// declares, and [`LumenVerifier::new`] honours it by rejecting every
    /// request rather than by falling back to an open one.
    pub required: bool,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
impl AuthConfig {
    pub fn open() -> Self {
        Self { required: false }
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

        if required {
            // Fail closed. A process asked to require an identity it cannot
            // verify has exactly one safe answer, and serving openly is not
            // it — the operator would get a working API and no indication
            // that the authentication they configured is absent.
            bail!(
                "LUMEN_AUTH=required (`spec.auth: required`) has no identity verification behind \
                 it: the bearer/identity registry was retired and the Kubernetes TokenReview \
                 verifier is not implemented yet. Refusing to start rather than serve an \
                 unauthenticated API. Set `spec.auth: disabled` (LUMEN_AUTH=disabled) if this \
                 deployment accepts an open API in the meantime."
            );
        }

        Ok(Self { required })
    }
}

/// Lumen's concrete verifier for the shared `service-auth` middleware: a
/// thin newtype over `service_auth::StaticRoleMapVerifier` holding no
/// credentials at all.
#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/logic/lumen-service-auth-convergence-delegate-middleware-to-shared-ver.md#logic
pub struct LumenVerifier(Arc<StaticRoleMapVerifier>);

/// @spec apps/lumen/tech-design/logic/lumen-service-auth-convergence-delegate-middleware-to-shared-ver.md#logic
impl LumenVerifier {
    pub fn new(cfg: Arc<AuthConfig>) -> Self {
        // The token map is empty and stays empty. Under `required` that means
        // no presented credential can ever resolve, so every request is
        // rejected — the same fail-closed answer `AuthConfig::from_env`
        // gives, for the state a caller built by hand.
        Self(Arc::new(StaticRoleMapVerifier::new(
            cfg.required,
            HashMap::new(),
        )))
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
/// thin newtype over the shared [`RoleMapPrincipal`] so [`ensure`](Self::ensure)
/// can map its rejection into lumen's own [`AuthErr`] / `ApiError` shape.
#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-auth-rs.md#source
pub struct AuthContext(RoleMapPrincipal);

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

    // Process-global env mutex shared across the env-mutating tests.
    use std::sync::Mutex;
    static AUTH_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_auth_env() {
        unsafe {
            std::env::remove_var("LUMEN_AUTH");
        }
    }

    #[test]
    fn auth_config_open_is_not_required() {
        assert!(!AuthConfig::open().required);
    }

    #[test]
    fn open_verifier_admits_an_unauthenticated_request_as_open() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::open()));
        let ctx = verifier.authenticate(&HeaderMap::new()).unwrap();
        assert!(ctx.ensure("any", Role::Admin).is_ok());
        assert_eq!(ctx.subject(), None);
    }

    /// #2871: the registry is gone, so there is no bearer secret left that
    /// resolves to a principal. A caller presenting one is rejected rather
    /// than silently promoted to the open principal.
    #[test]
    fn a_presented_bearer_no_longer_resolves_to_anything() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig::open()));
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer anything".parse().unwrap(),
        );
        assert!(matches!(
            verifier.authenticate(&headers).unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));
    }

    /// A hand-built `required` config has no registry to authenticate
    /// against, so it rejects every request — with and without a credential.
    /// The verifier never degrades to the open principal.
    #[test]
    fn a_required_verifier_rejects_every_request() {
        let verifier = LumenVerifier::new(Arc::new(AuthConfig { required: true }));
        assert!(matches!(
            verifier.authenticate(&HeaderMap::new()).unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer anything".parse().unwrap(),
        );
        assert!(matches!(
            verifier.authenticate(&headers).unwrap_err(),
            ServiceAuthError::Unauthenticated
        ));
    }

    #[test]
    fn auth_config_from_env_open_when_unset() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        let cfg = AuthConfig::from_env().unwrap();
        assert!(!cfg.required);
    }

    #[test]
    fn auth_config_from_env_accepts_both_disabled_spellings() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        for spelling in ["off", "disabled", "DISABLED", " off "] {
            clear_auth_env();
            unsafe {
                std::env::set_var("LUMEN_AUTH", spelling);
            }
            let cfg = AuthConfig::from_env()
                .unwrap_or_else(|e| panic!("`{spelling}` is a disabled spelling: {e:#}"));
            assert!(!cfg.required);
        }
        clear_auth_env();
    }

    /// R2: `required` fails closed at config load — before any listener is
    /// bound — and the message says why, not just that something is wrong.
    #[test]
    fn auth_config_required_fails_closed_naming_the_unimplemented_verifier() {
        let _g = AUTH_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_auth_env();
        unsafe {
            std::env::set_var("LUMEN_AUTH", "required");
        }
        let message = format!("{:#}", AuthConfig::from_env().unwrap_err());
        assert!(message.contains("LUMEN_AUTH=required"), "{message}");
        assert!(message.contains("TokenReview"), "{message}");
        assert!(message.contains("not implemented yet"), "{message}");
        assert!(message.contains("Refusing to start"), "{message}");
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
}
// CODEGEN-END
