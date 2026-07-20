// HANDWRITE-BEGIN gap="missing-generator:logic:defer-service-auth" tracker="#766" reason="Defer adapter for the shared bearer registry and per-queue role authorization."
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Result};
use service_auth::{
    AuditedRoleMapPrincipal, AuthError, ReloadableRoleMapVerifier, Role, TokenClaims,
    TracingAuthEventSink,
};

pub const AUTH_MODE_ENV: &str = "DEFER_AUTH";
pub const TOKEN_REGISTRY_FILE_ENV: &str = "DEFER_TOKEN_REGISTRY_FILE";
pub const LEGACY_TOKENS_ENV: &str = "DEFER_TOKENS";

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub required: bool,
    pub tokens: HashMap<String, TokenClaims>,
}

impl AuthConfig {
    pub fn open() -> Self {
        Self {
            required: false,
            tokens: HashMap::new(),
        }
    }

    pub fn resolve(
        mode: &str,
        registry_file: Option<&str>,
        legacy_tokens_json: Option<&str>,
    ) -> Result<Self> {
        let required = match mode.trim().to_ascii_lowercase().as_str() {
            "required" => true,
            "" | "off" | "disabled" => false,
            other => {
                bail!("{AUTH_MODE_ENV} must be `off`, `disabled`, or `required`; got `{other}`")
            }
        };
        let tokens = service_auth::load_registry(
            required,
            TOKEN_REGISTRY_FILE_ENV,
            registry_file,
            LEGACY_TOKENS_ENV,
            legacy_tokens_json,
        )?;
        Ok(Self { required, tokens })
    }

    pub fn verifier(&self) -> ReloadableRoleMapVerifier {
        ReloadableRoleMapVerifier::with_sink(
            self.required,
            self.tokens.clone(),
            Arc::new(TracingAuthEventSink),
        )
    }
}

pub fn authorize(
    principal: &AuditedRoleMapPrincipal,
    queue: &str,
    needed: Role,
) -> Result<(), AuthError> {
    principal.ensure(queue, needed).map_err(|denied| {
        AuthError::Forbidden(format!(
            "subject `{}` lacks {:?} on queue `{}`",
            denied.subject, denied.needed, denied.resource
        ))
    })
}
// HANDWRITE-END
