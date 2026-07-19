// HANDWRITE-BEGIN gap="sift-shared-bearer-auth" tracker="1604" reason="Adapt shared bearer-token verification to Sift environment configuration and data-plane middleware."
//! Sift's thin adapter over the shared static bearer-token role-map verifier.
//!
//! `SIFT_AUTH=required` protects only data-plane routes. Standard health,
//! readiness, metrics, OpenAPI, and docs routes remain operable for platform
//! probes. Production tokens are read from `SIFT_TOKEN_REGISTRY_FILE`; the
//! inline `SIFT_TOKENS` variable is retained for local development.

use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Result};
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use service_auth::{AuthError, RoleMapPrincipal, StaticRoleMapVerifier, TokenClaims, Verifier};

const TOKEN_REGISTRY_FILE_ENV: &str = "SIFT_TOKEN_REGISTRY_FILE";
const LEGACY_TOKENS_ENV: &str = "SIFT_TOKENS";

#[derive(Debug, Clone)]
pub struct SiftAuthConfig {
    pub required: bool,
    pub tokens: HashMap<String, TokenClaims>,
}

impl SiftAuthConfig {
    pub fn open() -> Self {
        Self {
            required: false,
            tokens: HashMap::new(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let required = match std::env::var("SIFT_AUTH") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "required" => true,
                "off" | "disabled" => false,
                other => bail!("SIFT_AUTH must be `off`, `disabled`, or `required`; got `{other}`"),
            },
            Err(std::env::VarError::NotPresent) => false,
            Err(error) => bail!("SIFT_AUTH must be valid UTF-8: {error}"),
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

#[derive(Debug, Clone)]
pub struct SiftVerifier(StaticRoleMapVerifier);

impl SiftVerifier {
    pub fn new(config: SiftAuthConfig) -> Self {
        Self(StaticRoleMapVerifier::new(config.required, config.tokens))
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self::new(SiftAuthConfig::from_env()?))
    }
}

impl Verifier for SiftVerifier {
    type Principal = RoleMapPrincipal;

    fn authenticate(&self, headers: &HeaderMap) -> Result<Self::Principal, AuthError> {
        self.0.authenticate(headers)
    }

    fn required(&self) -> bool {
        self.0.required()
    }
}

pub async fn auth_middleware(
    State(verifier): State<Arc<SiftVerifier>>,
    request: Request,
    next: Next,
) -> Response {
    service_auth::auth_middleware::<SiftVerifier>(State(verifier), request, next).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_config_has_no_tokens() {
        let config = SiftAuthConfig::open();
        assert!(!config.required);
        assert!(config.tokens.is_empty());
    }
}

// HANDWRITE-END
