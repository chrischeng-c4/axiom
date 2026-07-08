// HANDWRITE-BEGIN gap="missing-generator:logic:655920c0" tracker="pending-tracker" reason="relay's service-auth adapter: AuthConfig (RELAY_AUTH off|disabled|required mode parse + token-registry load via service_auth::load_registry with startup fail-fast naming RELAY_TOKEN_REGISTRY_FILE), StaticRoleMapVerifier construction (registry when required, open() when off), and the per-handler-group authorize(principal, subject, needed) helper mapping RoleMapDenied to the shared 403 forbidden shape."
//! relay's adoption of the shared `libs/service-auth` bearer contract (#1206).
//!
//! CONTRIBUTING § "Service auth — one Bearer-token contract": every
//! long-running service authenticates through `libs/service-auth`. relay uses
//! the standard registry verifier —
//! `service_auth::role_map::StaticRoleMapVerifier` — implementing the
//! archetype's `RELAY_AUTH=off|required` + `RELAY_TOKEN_REGISTRY_FILE` shape.
//! relay's auth is **all-or-nothing per deployment** (unlike keep's optional,
//! per-handler claim-check), so the blanket `service_auth::auth_middleware`
//! composes directly on the `/v1` data-plane router in [`crate::server`];
//! the probe surface (`/healthz` `/readyz` `/metrics` `/openapi.json`
//! `/docs`) never gets the layer and stays tokenless.
//!
//! This file owns relay's env/flag wiring and the per-handler-group
//! authorization policy. The role hierarchy, the wildcard `*` grant, the
//! registry-file loader, and the shared 401/403 `{error, message}` shape all
//! live in `libs/service-auth`.
//!
//! ## Configuration
//!
//! - `--auth` / `RELAY_AUTH` — `off` (default; tokenless dev) or `required`.
//! - `--token-registry-file` / `RELAY_TOKEN_REGISTRY_FILE` — the production
//!   registry, mounted from a Kubernetes Secret / Secret Manager projection.
//!   JSON: `{ "<token>": { "subject": "...", "roles":
//!   { "<subject>|*": "read|write|admin" } } }`.
//! - `RELAY_TOKENS` — legacy/dev inline JSON with the same shape (the shared
//!   loader's fallback; never the production path).
//!
//! Clients use `RELAY_URL` for routing plus `RELAY_TOKEN` for credentials and
//! send `Authorization: Bearer <token>`.
//!
//! ## Resource/role mapping (the `{subject}` path param is the resource)
//!
//! | handler group | needed role |
//! |---|---|
//! | `publish`, `publish-batch` | `write` |
//! | `consume`, `lease`, `ack`, `lease-batch`, `ack-batch`, `heartbeat`, `len` | `read` |
//!
//! Wildcard `*` grants cover every subject; `admin` ⊇ `write` ⊇ `read` per
//! [`Role::covers`].

use std::collections::HashMap;

use anyhow::{bail, Result};
use service_auth::{AuthError, Role, RoleMapPrincipal, StaticRoleMapVerifier, TokenClaims};

/// Auth-mode env var (`off`|`disabled`|`required`), surfaced as `--auth`.
pub const AUTH_MODE_ENV: &str = "RELAY_AUTH";
/// Token-registry-file env var, surfaced as `--token-registry-file`.
pub const TOKEN_REGISTRY_FILE_ENV: &str = "RELAY_TOKEN_REGISTRY_FILE";
/// Legacy/dev inline token-registry JSON (never the production path).
pub const LEGACY_TOKENS_ENV: &str = "RELAY_TOKENS";

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
    /// `RELAY_TOKEN_REGISTRY_FILE` — when auth is required but the registry
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

/// Per-handler-group authorization on the `{subject}` path param — the
/// service-auth split: the middleware authenticates, handlers authorize.
/// `publish`/`publish-batch` pass [`Role::Write`]; the consume family
/// (`consume`/`lease`/`ack`/`lease-batch`/`ack-batch`/`heartbeat`/`len`)
/// passes [`Role::Read`]. A denial maps the structured
/// [`service_auth::RoleMapDenied`] into the shared 403
/// `{"error": "forbidden", "message": ...}` shape (consistent with the #1205
/// `ApiErr` envelope family).
pub fn authorize(
    principal: &RoleMapPrincipal,
    subject: &str,
    needed: Role,
) -> Result<(), AuthError> {
    principal.ensure(subject, needed).map_err(|denied| {
        AuthError::Forbidden(format!(
            "subject `{}` lacks {:?} on `{}`",
            denied.subject, denied.needed, denied.resource
        ))
    })
}
// HANDWRITE-END
