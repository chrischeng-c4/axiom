// HANDWRITE-BEGIN gap="missing-generator:logic:f036c9bf" tracker="pending-tracker" reason="tape's service-auth adapter: AuthConfig (TAPE_AUTH off|disabled|required mode parse + token-registry load via service_auth::load_registry with startup fail-fast naming TAPE_TOKEN_REGISTRY_FILE and the TAPE_TOKENS legacy/dev inline fallback), StaticRoleMapVerifier construction (registry when required, open() when off), and the per-handler authorize(principal, topic, needed) helper mapping RoleMapDenied to the shared 403 forbidden shape."
//! tape's adoption of the shared `libs/service-auth` bearer contract (#1326).
//!
//! CONTRIBUTING § "Service auth — one Bearer-token contract": every
//! long-running service authenticates through `libs/service-auth`. tape uses
//! the standard registry verifier —
//! `service_auth::role_map::StaticRoleMapVerifier` — implementing the
//! archetype's `TAPE_AUTH=off|required` + `TAPE_TOKEN_REGISTRY_FILE` shape.
//! tape's auth is **all-or-nothing per deployment** (unlike keep's optional,
//! per-handler claim-check), so the blanket `service_auth::auth_middleware`
//! composes directly on the `/topics` data-plane router in
//! [`crate::server`]; the probe surface (`/healthz` `/readyz` `/metrics`
//! `/openapi.json` `/docs`) never gets the layer and stays tokenless.
//!
//! This file owns tape's env/flag wiring and the per-handler authorization
//! policy. The role hierarchy, the wildcard `*` grant, the registry-file
//! loader, and the shared 401/403 `{error, message}` shape all live in
//! `libs/service-auth`.
//!
//! ## Configuration
//!
//! - `--auth` / `TAPE_AUTH` — `off` (default; tokenless dev) or `required`.
//! - `--token-registry-file` / `TAPE_TOKEN_REGISTRY_FILE` — the production
//!   registry, mounted from a Kubernetes Secret / Secret Manager projection.
//!   JSON: `{ "<token>": { "subject": "...", "roles":
//!   { "<topic>|*": "read|write|admin" } } }`.
//! - `TAPE_TOKENS` — legacy/dev inline JSON with the same shape (the shared
//!   loader's fallback; never the production path).
//!
//! ## Resource/role mapping (the `{topic}` path param is the resource)
//!
//! | handler | needed role |
//! |---|---|
//! | `append` (producer side) | `write` |
//! | `replay`, `checkpoint_get`, `checkpoint_put` (consumer side) | `read` |
//!
//! `checkpoint_put` advances only the calling consumer's own replay cursor
//! -- it appends no new data to the topic -- so it sits in the `read` group
//! rather than a third tier, the same precedent relay set for its
//! consumer-local `ack`/`heartbeat`/`lease-batch`/`ack-batch` family (all
//! `read`, not `write`, even though `ack`/`heartbeat` mutate consumer-local
//! lease state).
//!
//! Wildcard `*` grants cover every topic; `admin` ⊇ `write` ⊇ `read` per
//! [`Role::covers`].

use std::collections::HashMap;

use anyhow::{bail, Result};
use service_auth::{AuthError, Role, RoleMapPrincipal, StaticRoleMapVerifier, TokenClaims};

/// Auth-mode env var (`off`|`disabled`|`required`), surfaced as `--auth`.
pub const AUTH_MODE_ENV: &str = "TAPE_AUTH";
/// Token-registry-file env var, surfaced as `--token-registry-file`.
pub const TOKEN_REGISTRY_FILE_ENV: &str = "TAPE_TOKEN_REGISTRY_FILE";
/// Legacy/dev inline token-registry JSON (never the production path).
pub const LEGACY_TOKENS_ENV: &str = "TAPE_TOKENS";

/// Resolved auth settings: the mode plus the token→claims registry.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub required: bool,
    pub tokens: HashMap<String, TokenClaims>,
}

impl AuthConfig {
    /// Open/dev config: auth off, no tokens — today's tokenless behavior.
    pub fn open() -> Self {
        Self {
            required: false,
            tokens: HashMap::new(),
        }
    }

    /// Resolve from the serve flags (each with env fallback): parse the mode,
    /// then load the registry through the shared loader. Fails fast — naming
    /// `TAPE_TOKEN_REGISTRY_FILE` — when auth is required but the registry
    /// file is missing, unparseable, or resolves empty: a server that can
    /// never authenticate anyone is a startup misconfiguration, not a
    /// per-request 401.
    pub fn resolve(
        mode: &str,
        registry_file: Option<&str>,
        legacy_tokens_json: Option<&str>,
    ) -> Result<Self> {
        let required = match mode.trim().to_ascii_lowercase().as_str() {
            "required" => true,
            "" | "off" | "disabled" => false,
            other => bail!(
                "{AUTH_MODE_ENV} (--auth) must be `off`, `disabled`, or `required`; got `{other}`"
            ),
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

    /// The verifier the data-plane `auth_middleware` runs: the shared static
    /// role-map over this registry (the `open()` shape when auth is off).
    pub fn verifier(&self) -> StaticRoleMapVerifier {
        StaticRoleMapVerifier::new(self.required, self.tokens.clone())
    }
}

/// Per-handler authorization on the `{topic}` path param — the
/// service-auth split: the middleware authenticates, handlers authorize.
/// `append` passes [`Role::Write`]; `replay`/`checkpoint_get`/
/// `checkpoint_put` pass [`Role::Read`]. A denial maps the structured
/// [`service_auth::RoleMapDenied`] into the shared 403
/// `{"error": "forbidden", "message": ...}` shape (consistent with the
/// `service_http::ApiErr` envelope family).
pub fn authorize(principal: &RoleMapPrincipal, topic: &str, needed: Role) -> Result<(), AuthError> {
    principal.ensure(topic, needed).map_err(|denied| {
        AuthError::Forbidden(format!(
            "topic `{}` lacks {:?} on `{}`",
            denied.subject, denied.needed, denied.resource
        ))
    })
}
// HANDWRITE-END
