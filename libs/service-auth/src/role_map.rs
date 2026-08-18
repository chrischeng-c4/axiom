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
//! - [`Registry`]: the two-namespace credential registry (#2678) — bearer
//!   secrets in `tokens`, provider-verified identities (Google emails) in
//!   `identities`. Kept disjoint so a bearer secret shaped like an email can
//!   never match an identity entry.
//! - [`load_registry_files`]: load a [`Registry`] from several projected
//!   files ([`RegistrySource`]), unioning them for a service that resolves both
//!   namespaces.
//! - [`load_registry_file`]: parse a [`Registry`] from a single registry-file path.
//! - [`load_registry`]: the bearer-only loader — a registry-file path
//!   (production, mounted from a Secret) or legacy inline JSON, failing fast
//!   when auth is required but the resolved registry ends up empty, and
//!   refusing a document that carries identity-keyed entries it could not
//!   resolve. Env-var *naming* stays the caller's concern — this fn only knows
//!   the resolved values plus the label strings to use in error context, so the
//!   wording stays byte-identical to whatever env vars a service actually reads.
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
pub enum Role {
    Read,
    Write,
    Admin,
}

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
pub struct TokenClaims {
    pub subject: String,
    /// `resource` → `Role`. The literal key `*` is a wildcard.
    #[serde(default)]
    pub roles: HashMap<String, Role>,
}

/// Section name for bearer-secret-keyed entries in a namespaced registry.
const TOKENS_SECTION: &str = "tokens";
/// Section name for identity-keyed entries in a namespaced registry.
const IDENTITIES_SECTION: &str = "identities";

/// A credential registry with two **disjoint** key namespaces (#2678).
///
/// - `tokens` is keyed by the bearer secret itself. The key *is* the
///   credential, which is why a file containing one has to be stored as a
///   secret.
/// - `identities` is keyed by a principal an identity provider has already
///   verified — a Google email, resolved by [`crate::gcp::GoogleVerifier`].
///   An email is public, so this half is ordinary configuration.
///
/// Keeping them apart is a security property, not tidiness: a shared map would
/// let a bearer secret that happens to be shaped like an email match an
/// identity entry and silently acquire its grants.
///
/// ## Document shapes
///
/// Namespaced (the shape that can carry identities):
///
/// ```json
/// { "tokens":     { "<secret>": { "subject": "svc", "roles": { "*": "read" } } },
///   "identities": { "a@b.com":  { "subject": "a",   "roles": { "*": "read" } } } }
/// ```
///
/// Flat (every existing service's file — every key is a bearer secret):
///
/// ```json
/// { "<secret>": { "subject": "svc", "roles": { "*": "read" } } }
/// ```
///
/// A document is read as namespaced when every top-level key is `tokens` or
/// `identities` **and** no top-level value is itself a claims object. The
/// second clause is what keeps a flat registry whose single secret is literally
/// spelled `tokens` from being misread as a section.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    /// Keyed by the bearer secret presented in `Authorization: Bearer …`.
    pub tokens: HashMap<String, TokenClaims>,
    /// Keyed by an identity an external provider verified (a Google email).
    pub identities: HashMap<String, TokenClaims>,
}

impl Registry {
    /// A bearer-only registry — the shape every service had before #2678.
    pub fn from_tokens(tokens: HashMap<String, TokenClaims>) -> Self {
        Self {
            tokens,
            identities: HashMap::new(),
        }
    }

    /// Total entries across both namespaces.
    pub fn len(&self) -> usize {
        self.tokens.len() + self.identities.len()
    }

    /// Whether the registry can authenticate nobody at all.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.identities.is_empty()
    }

    /// Parse either document shape. See the type docs for the discriminator.
    pub fn parse(json: &str) -> Result<Self> {
        let doc: serde_json::Value =
            serde_json::from_str(json).context("credential registry must be JSON")?;
        let namespaced = {
            let map = doc
                .as_object()
                .context("credential registry must be a JSON object")?;
            map.keys()
                .all(|key| key == TOKENS_SECTION || key == IDENTITIES_SECTION)
                && !map.values().any(|value| value.get("subject").is_some())
        };
        if !namespaced {
            return Ok(Self::from_tokens(serde_json::from_value(doc).context(
                "credential registry must map each bearer secret to its claims",
            )?));
        }
        let mut map = match doc {
            serde_json::Value::Object(map) => map,
            _ => unreachable!("checked as_object above"),
        };
        let mut section = |name: &str, what: &str| -> Result<HashMap<String, TokenClaims>> {
            match map.remove(name) {
                Some(value) => serde_json::from_value(value)
                    .with_context(|| format!("credential registry `{name}` must map {what}")),
                None => Ok(HashMap::new()),
            }
        };
        Ok(Self {
            tokens: section(TOKENS_SECTION, "each bearer secret to its claims")?,
            identities: section(IDENTITIES_SECTION, "each verified identity to its claims")?,
        })
    }

    /// Union another registry into this one, per namespace.
    ///
    /// The two namespaces have different confidentiality classes, so a
    /// deployment may well project them from different places — an
    /// `identities` map from a ConfigMap and a `tokens` map from a Secret
    /// (#2764). Merging is how those reunite into the one registry a request
    /// is resolved against.
    ///
    /// A key present in both inputs' *same* namespace is an error rather than
    /// a last-writer-wins overwrite: two sources disagreeing about one
    /// principal's grants leaves nobody able to say which grants are actually
    /// being served, which is the failure mode the split was meant to avoid.
    /// The same key appearing in *different* namespaces is not a collision —
    /// they are disjoint by construction (#2678, R1).
    pub fn try_merge(&mut self, other: Registry) -> Result<()> {
        merge_namespace(&mut self.tokens, other.tokens, TOKENS_SECTION)?;
        merge_namespace(&mut self.identities, other.identities, IDENTITIES_SECTION)
    }

    /// The first entry whose `subject` is one a service has reserved for its
    /// own use, as `(namespace, key, subject)`.
    ///
    /// A reserved subject is one the service itself presents — lumen's control
    /// plane names itself in every admin call it makes (#2679). A tenant
    /// registry that claims the same subject would make the operator's calls
    /// and a tenant's calls indistinguishable in audit output, so a registry
    /// carrying one is rejected rather than merged.
    pub fn reserved_subject_violation(
        &self,
        reserved: &[String],
    ) -> Option<(&'static str, String, String)> {
        let hit = |section: &'static str, entries: &HashMap<String, TokenClaims>| {
            entries
                .iter()
                .filter(|(_, claims)| reserved.iter().any(|r| r == &claims.subject))
                .map(|(key, claims)| (section, key.clone(), claims.subject.clone()))
                .min()
        };
        hit(TOKENS_SECTION, &self.tokens).or_else(|| hit(IDENTITIES_SECTION, &self.identities))
    }
}

fn merge_namespace(
    into: &mut HashMap<String, TokenClaims>,
    from: HashMap<String, TokenClaims>,
    section: &str,
) -> Result<()> {
    for (key, claims) in from {
        if let Some(previous) = into.get(&key) {
            // An `identities` key is a public email, and naming it is the
            // difference between a fixable message and a scavenger hunt. A
            // `tokens` key IS the bearer secret, so it is named by the subject
            // it grants instead — never by the key.
            let culprit = if section == IDENTITIES_SECTION {
                format!("`{key}`")
            } else {
                format!("the entry granting `{}`", previous.subject)
            };
            bail!(
                "credential registry sources disagree: `{section}` defines {culprit} more than \
                 once, so there is no way to say which grants are being served"
            );
        }
        into.insert(key, claims);
    }
    Ok(())
}

/// Load a credential registry from a registry-file path, for a service that
/// can resolve identity-keyed entries as well as bearer secrets.
///
/// Fails fast when `required` is true but the resolved registry ends up empty
/// in *both* namespaces (a server configured to mandate auth but unable to ever
/// authenticate anyone is a startup misconfiguration, not a per-request 401).
/// `registry_file_env` only words the error context; naming the actual env var
/// stays the caller's job, and a caller that knows a higher-level field
/// controls this should add that field to the error with `.context()`.
pub fn load_registry_file(
    required: bool,
    registry_file_env: &str,
    registry_file: Option<&str>,
) -> Result<Registry> {
    let registry = match registry_file {
        Some(path) if !path.trim().is_empty() => {
            let json = std::fs::read_to_string(path.trim())
                .with_context(|| format!("read {registry_file_env} `{}`", path.trim()))?;
            Registry::parse(&json)
                .with_context(|| format!("{registry_file_env} must contain JSON"))?
        }
        _ => Registry::default(),
    };
    if required && registry.is_empty() {
        bail!(
            "auth required but the credential registry is empty: point {registry_file_env} at a \
             file holding at least one `tokens` or `identities` entry"
        );
    }
    Ok(registry)
}

/// One place a registry may be projected from, and the env var that names it.
///
/// `env` only words the error messages; reading the variable stays the
/// caller's job so the message matches whatever the service calls it.
#[derive(Debug, Clone, Copy)]
pub struct RegistrySource<'a> {
    pub env: &'a str,
    pub path: Option<&'a str>,
}

/// Load a credential registry from several projected files at once, unioning
/// them.
///
/// The single-file [`load_registry_file`] assumes one Kubernetes object
/// carries the whole registry. That stops being true once the two namespaces
/// have different confidentiality classes: an `identities` map is ordinary
/// configuration (a ConfigMap), while a `tokens` map is a credential (a
/// Secret), and a deployment can reasonably have both (#2764). Each source is
/// parsed independently and [`Registry::try_merge`]d, so one malformed file
/// fails the load naming *that* file rather than silently serving a partial
/// registry.
///
/// Fails fast when `required` is true but every source is absent or empty: a
/// server told to mandate auth that can never authenticate anyone is a startup
/// misconfiguration, not a per-request 401.
pub fn load_registry_files(required: bool, sources: &[RegistrySource<'_>]) -> Result<Registry> {
    let mut registry = Registry::default();
    for source in sources {
        let Some(path) = source.path.map(str::trim).filter(|p| !p.is_empty()) else {
            continue;
        };
        let env = source.env;
        let json = std::fs::read_to_string(path)
            .with_context(|| format!("read {env} `{path}`"))?;
        let parsed = Registry::parse(&json).with_context(|| format!("{env} must contain JSON"))?;
        registry
            .try_merge(parsed)
            .with_context(|| format!("merging {env} `{path}`"))?;
    }
    if required && registry.is_empty() {
        let names = sources
            .iter()
            .map(|source| source.env)
            .collect::<Vec<_>>()
            .join(" or ");
        bail!(
            "auth required but the credential registry is empty: point {names} at a file holding \
             at least one `tokens` or `identities` entry"
        );
    }
    Ok(registry)
}

/// Load a **bearer-only** token registry: a registry-file path (preferred,
/// production shape) or legacy inline JSON, in that priority order. Accepts
/// either document shape [`Registry::parse`] understands, but rejects one
/// carrying identity-keyed entries — a service wired to this loader has no
/// identity verifier and could never resolve them, so silently dropping them
/// would present as an unexplained 401. Fails fast when `required` is true but
/// the resolved registry ends up empty (a server configured to mandate auth but
/// unable to ever authenticate anyone is a startup misconfiguration, not a
/// per-request 401). `registry_file_env`/`legacy_tokens_env` only word the
/// error context; naming the actual env vars stays the caller's job so the
/// message matches whatever the service calls them.
pub fn load_registry(
    required: bool,
    registry_file_env: &str,
    registry_file: Option<&str>,
    legacy_tokens_env: &str,
    legacy_tokens_json: Option<&str>,
) -> Result<HashMap<String, TokenClaims>> {
    let (registry, source_env) = match registry_file {
        Some(path) if !path.trim().is_empty() => {
            let json = std::fs::read_to_string(path.trim())
                .with_context(|| format!("read {registry_file_env} `{}`", path.trim()))?;
            let registry = Registry::parse(&json)
                .with_context(|| format!("{registry_file_env} must contain JSON"))?;
            (registry, registry_file_env)
        }
        _ => match legacy_tokens_json {
            Some(json) if !json.trim().is_empty() => {
                let registry = Registry::parse(json)
                    .with_context(|| format!("{legacy_tokens_env} must be JSON"))?;
                (registry, legacy_tokens_env)
            }
            _ => (Registry::default(), registry_file_env),
        },
    };
    if !registry.identities.is_empty() {
        bail!(
            "{source_env} carries `identities` entries, which need an identity provider to \
             resolve; this service authenticates bearer secrets only"
        );
    }
    if required && registry.tokens.is_empty() {
        bail!(
            "auth required but no tokens: set a non-empty {registry_file_env} or {legacy_tokens_env}"
        );
    }
    Ok(registry.tokens)
}

/// The resolved principal for a request: `Open` (auth disabled, no bearer
/// presented) or an authenticated token's claims.
#[derive(Debug, Clone)]
pub enum RoleMapPrincipal {
    /// Auth is disabled and no bearer was presented. Treated as full admin
    /// in development; production should run with auth required.
    Open,
    Token(TokenClaims),
}

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
pub struct RoleMapDenied {
    pub subject: String,
    pub needed: Role,
    pub resource: String,
}

/// A [`Verifier`] over a static, config-driven token→claims registry — the
/// archetype's `<SVC>_AUTH=off|required` shape.
#[derive(Debug, Clone)]
pub struct StaticRoleMapVerifier {
    required: bool,
    tokens: HashMap<String, TokenClaims>,
}

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

    // -- #2678: the two namespaces ----------------------------------------

    const NAMESPACED: &str = r#"{
        "tokens":     { "s3cret":  { "subject": "svc", "roles": { "products": "write" } } },
        "identities": { "a@b.com": { "subject": "dev", "roles": { "products": "read"  } } }
    }"#;

    #[test]
    fn namespaced_document_lands_each_entry_in_its_own_namespace() {
        let registry = Registry::parse(NAMESPACED).unwrap();
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.tokens["s3cret"].subject, "svc");
        assert_eq!(registry.identities["a@b.com"].subject, "dev");
        // The whole point: neither key resolves from the other's map.
        assert!(!registry.tokens.contains_key("a@b.com"));
        assert!(!registry.identities.contains_key("s3cret"));
    }

    #[test]
    fn a_flat_document_is_read_as_bearer_secrets() {
        let registry =
            Registry::parse(r#"{"s3cret":{"subject":"svc","roles":{"*":"read"}}}"#).unwrap();
        assert_eq!(registry.tokens.len(), 1);
        assert!(registry.identities.is_empty());
    }

    /// The discriminator is "all keys are section names **and** no top-level
    /// value is a claims object", not just the first clause — otherwise a flat
    /// registry whose bearer secret is literally the word `tokens` would be
    /// misread as an empty section and silently authenticate nobody.
    #[test]
    fn a_flat_document_whose_secret_is_spelled_tokens_is_still_flat() {
        let registry =
            Registry::parse(r#"{"tokens":{"subject":"svc","roles":{"*":"read"}}}"#).unwrap();
        assert_eq!(registry.tokens["tokens"].subject, "svc");
        assert!(registry.identities.is_empty());
    }

    #[test]
    fn a_partial_namespaced_document_leaves_the_absent_section_empty() {
        let registry = Registry::parse(r#"{"identities":{"a@b.com":{"subject":"dev","roles":{}}}}"#)
            .unwrap();
        assert!(registry.tokens.is_empty());
        assert_eq!(registry.identities.len(), 1);
    }

    /// A bearer-only service that was handed identities cannot resolve them.
    /// Dropping them silently would present to the operator as an unexplained
    /// 401 on a credential the registry appears to grant.
    #[test]
    fn bearer_only_loader_rejects_a_document_carrying_identities() {
        let dir = std::env::temp_dir().join("service-auth-2678-identities");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("registry.json");
        std::fs::write(&path, NAMESPACED).unwrap();

        let err = load_registry(
            true,
            "REGISTRY_FILE",
            Some(path.to_str().unwrap()),
            "LEGACY_TOKENS",
            None,
        )
        .unwrap_err();

        let message = err.to_string();
        assert!(message.contains("REGISTRY_FILE"), "{message}");
        assert!(message.contains("identities"), "{message}");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn identity_only_registry_satisfies_required_for_an_identity_aware_service() {
        let dir = std::env::temp_dir().join("service-auth-2678-identity-only");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("registry.json");
        std::fs::write(
            &path,
            r#"{"identities":{"a@b.com":{"subject":"dev","roles":{"*":"read"}}}}"#,
        )
        .unwrap();

        let registry = load_registry_file(true, "REGISTRY_FILE", Some(path.to_str().unwrap()))
            .expect("an identity-only registry can authenticate someone");
        assert_eq!(registry.identities.len(), 1);
        assert!(registry.tokens.is_empty());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn identity_aware_loader_fails_fast_when_required_and_both_namespaces_are_empty() {
        let err = load_registry_file(true, "REGISTRY_FILE", None).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("REGISTRY_FILE"), "{message}");
        assert!(message.contains("identities"), "{message}");
    }

    // -- #2764: two sources, two confidentiality classes -------------------

    /// A scratch directory per test. Sharing one would let a parallel test's
    /// cleanup delete a file this one is mid-read on.
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("service-auth-2764-{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &std::path::Path, file: &str, body: &str) -> String {
        let path = dir.join(file);
        std::fs::write(&path, body).unwrap();
        path.to_str().unwrap().to_owned()
    }

    /// #2764's structural claim. The identity map is a ConfigMap and the
    /// bearer secrets are a Secret; two Kubernetes objects cannot share one
    /// mount path, so the loader must union two files. If it could not, the
    /// only way to serve both classes would be to put the plaintext identity
    /// map back inside the Secret — exactly the coupling this work removes.
    #[test]
    fn two_files_union_into_one_registry_each_keeping_its_own_namespace() {
        let dir = scratch("union");
        let identities = write(
            &dir,
            "identities.json",
            r#"{"identities":{"a@b.com":{"subject":"dev","roles":{"products":"read"}}}}"#,
        );
        let tokens = write(
            &dir,
            "token-registry.json",
            r#"{"tokens":{"s3cret":{"subject":"svc","roles":{"products":"write"}}}}"#,
        );

        let registry = load_registry_files(
            true,
            &[
                RegistrySource {
                    env: "IDENTITY_FILE",
                    path: Some(&identities),
                },
                RegistrySource {
                    env: "TOKEN_FILE",
                    path: Some(&tokens),
                },
            ],
        )
        .expect("two sources union");

        assert_eq!(registry.identities["a@b.com"].subject, "dev");
        assert_eq!(registry.tokens["s3cret"].subject, "svc");
        // Still disjoint after the merge — the union is per namespace.
        assert!(!registry.identities.contains_key("s3cret"));
        assert!(!registry.tokens.contains_key("a@b.com"));
    }

    /// Last-writer-wins would reproduce the failure #2764 quotes as the reason
    /// to delete the mutual-exclusion CEL rule: no way to tell which registry
    /// is actually being served. Two sources disagreeing about one principal's
    /// grants is a deployment mistake, and it fails loudly.
    #[test]
    fn a_key_claimed_by_two_sources_is_an_error_not_a_silent_overwrite() {
        let dir = scratch("collision");
        let first = write(
            &dir,
            "a.json",
            r#"{"identities":{"a@b.com":{"subject":"dev","roles":{"products":"read"}}}}"#,
        );
        let second = write(
            &dir,
            "b.json",
            r#"{"identities":{"a@b.com":{"subject":"dev","roles":{"products":"admin"}}}}"#,
        );

        let err = load_registry_files(
            true,
            &[
                RegistrySource {
                    env: "FIRST",
                    path: Some(&first),
                },
                RegistrySource {
                    env: "SECOND",
                    path: Some(&second),
                },
            ],
        )
        .unwrap_err();

        let message = format!("{err:#}");
        assert!(message.contains("a@b.com"), "{message}");
        assert!(message.contains("SECOND"), "{message}");
    }

    /// The collision message is read by whoever is debugging the deployment,
    /// which usually means it lands in a log aggregator. A `tokens` key is the
    /// bearer secret itself, so it is named by the subject it grants.
    #[test]
    fn a_token_collision_names_the_subject_and_never_the_secret() {
        let mut registry =
            Registry::parse(r#"{"tokens":{"s3cret":{"subject":"svc","roles":{}}}}"#).unwrap();
        let other =
            Registry::parse(r#"{"tokens":{"s3cret":{"subject":"svc","roles":{}}}}"#).unwrap();

        let message = registry.try_merge(other).unwrap_err().to_string();
        assert!(message.contains("svc"), "{message}");
        assert!(
            !message.contains("s3cret"),
            "the collision message leaked a bearer secret: {message}"
        );
    }

    /// The same key in *different* namespaces is not a collision: one is a
    /// secret and one is an email, and they are resolved by different lookups.
    #[test]
    fn the_same_key_in_two_namespaces_is_not_a_collision() {
        let mut registry =
            Registry::parse(r#"{"tokens":{"shared":{"subject":"svc","roles":{}}}}"#).unwrap();
        let other =
            Registry::parse(r#"{"identities":{"shared":{"subject":"dev","roles":{}}}}"#).unwrap();
        registry.try_merge(other).expect("different namespaces");
        assert_eq!(registry.tokens["shared"].subject, "svc");
        assert_eq!(registry.identities["shared"].subject, "dev");
    }

    /// An absent source is absent, not empty. A service configured with only
    /// an identity map must not be told its registry file failed to read.
    #[test]
    fn unset_and_blank_sources_are_skipped() {
        let dir = scratch("skip");
        let identities = write(
            &dir,
            "identities.json",
            r#"{"identities":{"a@b.com":{"subject":"dev","roles":{}}}}"#,
        );

        let registry = load_registry_files(
            true,
            &[
                RegistrySource {
                    env: "IDENTITY_FILE",
                    path: Some(&identities),
                },
                RegistrySource {
                    env: "TOKEN_FILE",
                    path: None,
                },
                RegistrySource {
                    env: "LEGACY_FILE",
                    path: Some("   "),
                },
            ],
        )
        .expect("one configured source is enough");
        assert_eq!(registry.len(), 1);
    }

    /// Fail-fast names every source the operator could have set, because the
    /// mistake is usually "I set the other one".
    #[test]
    fn required_with_no_source_at_all_names_every_env_var() {
        let err = load_registry_files(
            true,
            &[
                RegistrySource {
                    env: "IDENTITY_FILE",
                    path: None,
                },
                RegistrySource {
                    env: "TOKEN_FILE",
                    path: None,
                },
            ],
        )
        .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("IDENTITY_FILE"), "{message}");
        assert!(message.contains("TOKEN_FILE"), "{message}");
    }

    /// #2679's half of the invariant. The control plane presents an identity
    /// of its own; a tenant who could grant that same subject to a credential
    /// they hold would be impersonating the operator, and every audit line
    /// would name the operator rather than them.
    #[test]
    fn a_registry_claiming_a_reserved_subject_is_reported_with_its_section_and_key() {
        let registry = Registry::parse(
            r#"{"identities":{"tenant@b.com":{"subject":"lumen-control-plane","roles":{}}}}"#,
        )
        .unwrap();

        let (section, key, subject) = registry
            .reserved_subject_violation(&["lumen-control-plane".to_owned()])
            .expect("the reserved subject is claimed");
        assert_eq!(section, "identities");
        assert_eq!(key, "tenant@b.com");
        assert_eq!(subject, "lumen-control-plane");

        assert!(registry
            .reserved_subject_violation(&["someone-else".to_owned()])
            .is_none());
        assert!(registry.reserved_subject_violation(&[]).is_none());
    }
}
// CODEGEN-END
