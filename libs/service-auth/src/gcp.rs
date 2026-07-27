// HANDWRITE-BEGIN gap="missing-generator:logic:google-identity" tracker="#2677" reason="Google ID-token verification against a cached JWKS, access-token introspection with a bounded cache, credential-shape discrimination, and the shared resolution tail into the existing role map."
//! Google identity verification — "IAM decides who you are, the product
//! decides what you may do".
//!
//! This module resolves a Google-issued credential to a **verified email** and
//! then hands that email to the unmodified role map. [`Role`], [`covers`], and
//! [`ensure`] are untouched: authentication gains a new source, authorization
//! does not change.
//!
//! ## Two credentials, two verification models
//!
//! They are different artifacts from different endpoints, and conflating them
//! produces a 401 that looks like a service bug:
//!
//! | | ID token | Access token |
//! |---|---|---|
//! | Shape | RS256 JWT, three segments, carries `kid` | opaque `ya29.*` |
//! | Verified by | offline, against Google's published JWKS | calling Google's introspection endpoint |
//! | `aud` | bound to a requested audience | the client ID that minted it |
//! | Mintable by a plain user account | **no** | yes |
//! | Mintable by a service account | yes | yes |
//!
//! The last two rows are why both paths exist rather than one. A developer
//! cannot mint a custom-audience ID token from a plain Google account, and a
//! workload should not put a Google round trip in its request path. So a
//! service takes the offline path and a human takes the introspection path —
//! the same split Cloud SQL IAM Database Authentication uses.
//!
//! ## Which path a credential takes
//!
//! Selection is by shape, never by trying each in turn ([`classify`]):
//!
//! ```text
//! three dot-separated segments, RS256, carries kid  ->  offline JWKS path
//! anything else                                     ->  registry lookup,
//!                                                       then introspection
//! ```
//!
//! Registry-first on the opaque branch is deliberate: a pre-shared bearer
//! secret is a local map hit, and it must not acquire a network round trip
//! just because Google identities became possible.
//!
//! ## Staying off the network
//!
//! Steady state makes no upstream call. JWKS keys are cached and only
//! refetched when a `kid` misses — Google rotates signing keys, so pinning one
//! would mean a total outage at rotation — and that refetch is rate-limited so
//! fabricated `kid` values cannot be amplified into a flood against Google.
//! Introspection results are cached for `min(expires_in, ceiling)`.
//!
//! ## Reachability is not rejection
//!
//! [`GoogleAuthError::IntrospectionUnavailable`] and
//! [`GoogleAuthError::SigningKeyUnavailable`] surface as
//! [`AuthError::Unavailable`] (503), never as 401. An upstream outage that
//! presents to the caller as "your credential is invalid" is an unfixable
//! support call.
//!
//! [`Role`]: crate::role_map::Role
//! [`covers`]: crate::role_map::Role::covers
//! [`ensure`]: crate::role_map::RoleMapPrincipal::ensure

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use axum::http::HeaderMap;
use jsonwebtoken::{
    decode, decode_header, errors::ErrorKind as JwtErrorKind, jwk::JwkSet, Algorithm, DecodingKey,
    Validation,
};
use serde::{Deserialize, Deserializer};

use crate::async_verifier::AsyncVerifier;
use crate::error::AuthError;
use crate::middleware::bearer_token;
use crate::reload::ReloadableRoleMapVerifier;
use crate::role_map::RoleMapPrincipal;

/// Both issuer spellings Google emits for ID tokens. Pinned, not configurable:
/// there is no second issuer to validate an abstraction against.
pub const GOOGLE_ISSUERS: [&str; 2] = ["https://accounts.google.com", "accounts.google.com"];

/// Google's published JWKS for ID-token signing keys.
pub const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

/// Google's access-token introspection endpoint.
pub const GOOGLE_TOKENINFO_URL: &str = "https://oauth2.googleapis.com/tokeninfo";

/// Floor between JWKS refetches. A caller presenting fabricated `kid` values
/// gets at most one upstream fetch per window, not one per request.
pub const DEFAULT_JWKS_REFETCH_MIN_INTERVAL: Duration = Duration::from_secs(60);

/// Ceiling on how long an introspection result is trusted.
///
/// This is the revocation-latency knob. Caching to the token's own
/// `expires_in` means a revoked credential keeps working for up to its
/// remaining hour; a short ceiling narrows that window at the cost of more
/// calls to Google.
pub const DEFAULT_INTROSPECTION_TTL_CEILING: Duration = Duration::from_secs(300);

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Why a Google credential did not resolve to an authorized principal.
///
/// Every variant is distinguishable on purpose: "expired" and "wrong
/// audience" and "we could not reach Google" are three different operator
/// actions, and collapsing them into one 401 is what makes an auth layer
/// unsupportable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoogleAuthError {
    /// Not a well-formed JWT, or a JWT without the `kid` that key rotation
    /// makes mandatory.
    MalformedToken(String),
    /// The token is well formed but failed validation.
    Invalid(InvalidReason),
    /// The `kid` is absent from the JWKS and a refetch is rate-limited or did
    /// not produce it. Distinct from [`Self::SigningKeyUnavailable`]: here
    /// Google answered, and the key genuinely is not published.
    UnknownSigningKey { kid: String },
    /// The JWKS could not be fetched or parsed. An upstream problem, not a
    /// caller problem.
    SigningKeyUnavailable(String),
    /// The credential verified but carries no `email` claim — an ID token
    /// minted without `--include-email`.
    EmailMissing,
    /// The credential carries an email Google has not verified.
    EmailUnverified,
    /// The introspection endpoint answered that the token is not valid.
    Rejected,
    /// The introspection endpoint could not be reached. Distinct from
    /// [`Self::Rejected`], which is the whole point of the variant.
    IntrospectionUnavailable(String),
    /// No introspector is configured, so an opaque credential that missed the
    /// registry has nowhere left to go.
    IntrospectionNotConfigured,
    /// Authentication succeeded and authorization did not: a verified identity
    /// that no registry entry grants anything to.
    NotInRegistry,
}

/// The specific validation that failed. Separate from [`GoogleAuthError`] so
/// the five negatives an operator actually hits stay individually assertable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidReason {
    Audience,
    Issuer,
    Expired,
    Signature,
}

impl fmt::Display for GoogleAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedToken(why) => write!(f, "malformed credential: {why}"),
            Self::Invalid(InvalidReason::Audience) => {
                write!(f, "token was minted for a different audience")
            }
            Self::Invalid(InvalidReason::Issuer) => write!(f, "token issuer is not Google"),
            Self::Invalid(InvalidReason::Expired) => write!(f, "token has expired"),
            Self::Invalid(InvalidReason::Signature) => write!(f, "token signature is not valid"),
            Self::UnknownSigningKey { kid } => {
                write!(f, "signing key `{kid}` is not published by Google")
            }
            Self::SigningKeyUnavailable(why) => write!(f, "could not obtain signing keys: {why}"),
            Self::EmailMissing => write!(f, "token carries no email claim"),
            Self::EmailUnverified => write!(f, "token carries an unverified email"),
            Self::Rejected => write!(f, "credential was rejected by Google"),
            Self::IntrospectionUnavailable(why) => {
                write!(f, "could not reach Google to check the credential: {why}")
            }
            Self::IntrospectionNotConfigured => {
                write!(f, "no access-token introspection is configured")
            }
            Self::NotInRegistry => write!(f, "identity is not granted anything by the registry"),
        }
    }
}

impl std::error::Error for GoogleAuthError {}

impl GoogleAuthError {
    /// Whether this is an upstream failure rather than a verdict on the
    /// caller's credential. Drives the 503-vs-401 split.
    pub fn is_upstream_failure(&self) -> bool {
        matches!(
            self,
            Self::SigningKeyUnavailable(_) | Self::IntrospectionUnavailable(_)
        )
    }
}

impl From<GoogleAuthError> for AuthError {
    fn from(error: GoogleAuthError) -> Self {
        if error.is_upstream_failure() {
            // The message is built from upstream error text with the URL
            // stripped, so it cannot carry the presented credential.
            AuthError::Unavailable(error.to_string())
        } else {
            AuthError::Unauthenticated
        }
    }
}

fn classify_jwt_error(error: &jsonwebtoken::errors::Error) -> GoogleAuthError {
    match error.kind() {
        JwtErrorKind::InvalidAudience => GoogleAuthError::Invalid(InvalidReason::Audience),
        JwtErrorKind::InvalidIssuer => GoogleAuthError::Invalid(InvalidReason::Issuer),
        JwtErrorKind::ExpiredSignature => GoogleAuthError::Invalid(InvalidReason::Expired),
        JwtErrorKind::InvalidSignature => GoogleAuthError::Invalid(InvalidReason::Signature),
        other => GoogleAuthError::MalformedToken(format!("{other:?}")),
    }
}

// ---------------------------------------------------------------------------
// Injection seams: clock, JWKS source, introspection
// ---------------------------------------------------------------------------

/// Wall clock, injected so cache expiry and refetch rate limiting are testable
/// without sleeping.
///
/// JWT `exp` validation is **not** routed through this — `jsonwebtoken` reads
/// the system clock itself, so expiry tests mint a token with a past `exp`
/// rather than moving a fake clock.
pub trait Clock: Send + Sync {
    fn now_unix(&self) -> u64;
}

/// The production clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default()
    }
}

/// Where ID-token signing keys come from.
#[async_trait]
pub trait JwksSource: Send + Sync {
    /// Fetch the current key set. The error is a message, not a typed cause:
    /// every failure here means the same thing to a caller — upstream is not
    /// answering — and the detail exists only for the operator's log.
    async fn fetch(&self) -> Result<JwkSet, String>;
}

/// What Google's introspection endpoint reports about an access token.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct IntrospectedToken {
    #[serde(default)]
    pub email: Option<String>,
    /// Google sends this as the **string** `"true"`, not a JSON boolean.
    #[serde(default, deserialize_with = "lenient_bool")]
    pub email_verified: bool,
    /// Seconds of remaining life. Also sent as a string.
    #[serde(default, deserialize_with = "lenient_u64")]
    pub expires_in: u64,
}

/// Resolves an opaque access token by asking Google.
#[async_trait]
pub trait AccessTokenIntrospection: Send + Sync {
    /// `Ok(Some(_))` — Google answered and the token is live.
    /// `Ok(None)` — Google answered and the token is not valid.
    /// `Err(_)` — Google did not answer. These three are different outcomes
    /// and the type keeps them different.
    async fn introspect(&self, token: &str) -> Result<Option<IntrospectedToken>, String>;
}

fn lenient_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        Str(String),
    }
    Ok(match BoolOrString::deserialize(deserializer)? {
        BoolOrString::Bool(value) => value,
        BoolOrString::Str(value) => matches!(value.trim(), "true" | "True" | "TRUE" | "1"),
    })
}

fn lenient_u64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrString {
        Num(u64),
        Str(String),
    }
    Ok(match NumOrString::deserialize(deserializer)? {
        NumOrString::Num(value) => value,
        NumOrString::Str(value) => value.trim().parse().unwrap_or(0),
    })
}

// ---------------------------------------------------------------------------
// HTTP implementations
// ---------------------------------------------------------------------------

/// Fetches Google's published JWKS over HTTPS.
#[derive(Debug, Clone)]
pub struct HttpJwksSource {
    client: reqwest::Client,
    url: String,
}

impl HttpJwksSource {
    pub fn new(client: reqwest::Client, url: impl Into<String>) -> Self {
        Self {
            client,
            url: url.into(),
        }
    }

    pub fn google(client: reqwest::Client) -> Self {
        Self::new(client, GOOGLE_JWKS_URL)
    }
}

#[async_trait]
impl JwksSource for HttpJwksSource {
    async fn fetch(&self) -> Result<JwkSet, String> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| e.without_url().to_string())?;
        if !response.status().is_success() {
            return Err(format!("JWKS endpoint returned {}", response.status()));
        }
        response
            .json::<JwkSet>()
            .await
            .map_err(|e| format!("JWKS response is not a key set: {}", e.without_url()))
    }
}

/// Calls Google's `tokeninfo` endpoint.
///
/// Every error is passed through [`reqwest::Error::without_url`] before it
/// becomes a string. The access token travels in the query string, so an error
/// that echoed its URL would put the caller's live credential into the
/// service's logs.
#[derive(Debug, Clone)]
pub struct HttpAccessTokenIntrospection {
    client: reqwest::Client,
    url: String,
}

impl HttpAccessTokenIntrospection {
    pub fn new(client: reqwest::Client, url: impl Into<String>) -> Self {
        Self {
            client,
            url: url.into(),
        }
    }

    pub fn google(client: reqwest::Client) -> Self {
        Self::new(client, GOOGLE_TOKENINFO_URL)
    }
}

#[async_trait]
impl AccessTokenIntrospection for HttpAccessTokenIntrospection {
    async fn introspect(&self, token: &str) -> Result<Option<IntrospectedToken>, String> {
        let response = self
            .client
            .get(&self.url)
            .query(&[("access_token", token)])
            .send()
            .await
            .map_err(|e| e.without_url().to_string())?;

        // 400 is Google's "this token is not valid" answer — a verdict, not an
        // outage, so it must not become IntrospectionUnavailable.
        if response.status() == reqwest::StatusCode::BAD_REQUEST
            || response.status() == reqwest::StatusCode::UNAUTHORIZED
        {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(format!("tokeninfo returned {}", response.status()));
        }
        response
            .json::<IntrospectedToken>()
            .await
            .map(Some)
            .map_err(|e| format!("tokeninfo response was not understood: {}", e.without_url()))
    }
}

// ---------------------------------------------------------------------------
// JWKS cache
// ---------------------------------------------------------------------------

#[derive(Default)]
struct JwksState {
    keys: Option<JwkSet>,
    last_fetch_unix: Option<u64>,
}

/// Caches Google's signing keys and refetches, at a bounded rate, when a `kid`
/// misses.
pub struct JwksCache {
    source: Arc<dyn JwksSource>,
    clock: Arc<dyn Clock>,
    min_refetch_interval: Duration,
    state: RwLock<JwksState>,
}

impl fmt::Debug for JwksCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwksCache")
            .field("min_refetch_interval", &self.min_refetch_interval)
            .finish_non_exhaustive()
    }
}

impl JwksCache {
    pub fn new(
        source: Arc<dyn JwksSource>,
        clock: Arc<dyn Clock>,
        min_refetch_interval: Duration,
    ) -> Self {
        Self {
            source,
            clock,
            min_refetch_interval,
            state: RwLock::new(JwksState::default()),
        }
    }

    fn cached_key(&self, kid: &str) -> Result<Option<DecodingKey>, GoogleAuthError> {
        let state = self
            .state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(keys) = state.keys.as_ref() else {
            return Ok(None);
        };
        let Some(jwk) = keys.find(kid) else {
            return Ok(None);
        };
        DecodingKey::from_jwk(jwk).map(Some).map_err(|e| {
            GoogleAuthError::SigningKeyUnavailable(format!(
                "JWKS entry for `{kid}` is not a usable key: {e}"
            ))
        })
    }

    /// Resolve a `kid` to a decoding key, refetching at most once per window.
    ///
    /// The lock is never held across the fetch, so a slow upstream cannot
    /// stall requests that hit the cache. Two concurrent misses can therefore
    /// both fetch; that is bounded and preferable to serializing every request
    /// behind one lock.
    pub async fn decoding_key(&self, kid: &str) -> Result<DecodingKey, GoogleAuthError> {
        if let Some(key) = self.cached_key(kid)? {
            return Ok(key);
        }

        let now = self.clock.now_unix();
        {
            let state = self
                .state
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(last) = state.last_fetch_unix {
                if now.saturating_sub(last) < self.min_refetch_interval.as_secs() {
                    return Err(GoogleAuthError::UnknownSigningKey {
                        kid: kid.to_string(),
                    });
                }
            }
        }

        let fetched = self.source.fetch().await;
        {
            let mut state = self
                .state
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.last_fetch_unix = Some(now);
            match fetched {
                Ok(keys) => state.keys = Some(keys),
                Err(why) => return Err(GoogleAuthError::SigningKeyUnavailable(why)),
            }
        }

        self.cached_key(kid)?
            .ok_or_else(|| GoogleAuthError::UnknownSigningKey {
                kid: kid.to_string(),
            })
    }
}

// ---------------------------------------------------------------------------
// Introspection cache
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct CachedIdentity {
    email: String,
    expires_at_unix: u64,
}

/// Which shape a presented credential has, and therefore which path verifies
/// it. Selection never falls through from one path to the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Credential {
    /// A three-segment RS256 JWT carrying a `kid` — the offline path.
    GoogleIdToken,
    /// Anything else — registry lookup, then introspection.
    Opaque,
}

/// Classify a presented credential by shape alone.
pub fn classify(token: &str) -> Credential {
    if token.split('.').count() != 3 {
        return Credential::Opaque;
    }
    match decode_header(token) {
        Ok(header) if header.kid.is_some() && header.alg == Algorithm::RS256 => {
            Credential::GoogleIdToken
        }
        _ => Credential::Opaque,
    }
}

// ---------------------------------------------------------------------------
// The verifier
// ---------------------------------------------------------------------------

/// Tunables for [`GoogleVerifier`].
#[derive(Debug, Clone)]
pub struct GoogleAuthConfig {
    /// Audiences an ID token may be minted for. An empty list is rejected at
    /// construction: an ID-token verifier that accepts any audience accepts
    /// tokens minted for someone else's service.
    pub audiences: Vec<String>,
    pub jwks_refetch_min_interval: Duration,
    pub introspection_ttl_ceiling: Duration,
}

impl GoogleAuthConfig {
    pub fn new(audiences: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            audiences: audiences.into_iter().map(Into::into).collect(),
            jwks_refetch_min_interval: DEFAULT_JWKS_REFETCH_MIN_INTERVAL,
            introspection_ttl_ceiling: DEFAULT_INTROSPECTION_TTL_CEILING,
        }
    }
}

/// A [`AsyncVerifier`] that accepts Google identities and pre-shared bearer
/// secrets, and authorizes both through one unmodified role map.
pub struct GoogleVerifier {
    required: bool,
    registry: Arc<ReloadableRoleMapVerifier>,
    validation: Validation,
    jwks: JwksCache,
    introspection: Option<Arc<dyn AccessTokenIntrospection>>,
    introspection_ttl_ceiling: Duration,
    introspection_cache: RwLock<HashMap<String, CachedIdentity>>,
    clock: Arc<dyn Clock>,
}

impl fmt::Debug for GoogleVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No cache contents: its keys are live credentials.
        f.debug_struct("GoogleVerifier")
            .field("required", &self.required)
            .field("introspection", &self.introspection.is_some())
            .field("registry_revision", &self.registry.revision())
            .finish_non_exhaustive()
    }
}

impl GoogleVerifier {
    /// Build a verifier against the real Google endpoints.
    pub fn google(
        required: bool,
        registry: Arc<ReloadableRoleMapVerifier>,
        config: GoogleAuthConfig,
    ) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self::with_sources(
            required,
            registry,
            config,
            Arc::new(HttpJwksSource::google(client.clone())),
            Some(Arc::new(HttpAccessTokenIntrospection::google(client))),
            Arc::new(SystemClock),
        ))
    }

    /// Build a verifier over injected sources — the constructor tests use, and
    /// the one a service uses to disable the introspection path by passing
    /// `None`.
    pub fn with_sources(
        required: bool,
        registry: Arc<ReloadableRoleMapVerifier>,
        config: GoogleAuthConfig,
        jwks: Arc<dyn JwksSource>,
        introspection: Option<Arc<dyn AccessTokenIntrospection>>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&config.audiences);
        validation.set_issuer(&GOOGLE_ISSUERS);
        // Validated by default; stated so the intent survives a refactor.
        validation.validate_exp = true;

        Self {
            required,
            registry,
            validation,
            jwks: JwksCache::new(jwks, Arc::clone(&clock), config.jwks_refetch_min_interval),
            introspection,
            introspection_ttl_ceiling: config.introspection_ttl_ceiling,
            introspection_cache: RwLock::new(HashMap::new()),
            clock,
        }
    }

    /// Verify a Google ID token offline and return its verified email.
    pub async fn verify_id_token(&self, token: &str) -> Result<String, GoogleAuthError> {
        let header = decode_header(token)
            .map_err(|e| GoogleAuthError::MalformedToken(format!("{:?}", e.kind())))?;
        let kid = header.kid.ok_or_else(|| {
            GoogleAuthError::MalformedToken(
                "no kid; key rotation would be unresolvable without one".to_string(),
            )
        })?;
        let key = self.jwks.decoding_key(&kid).await?;
        let data = decode::<GoogleIdClaims>(token, &key, &self.validation)
            .map_err(|e| classify_jwt_error(&e))?;
        verified_email(data.claims.email, data.claims.email_verified)
    }

    /// Resolve an opaque access token to a verified email by asking Google,
    /// serving a cached answer when one is still live.
    pub async fn introspect_access_token(&self, token: &str) -> Result<String, GoogleAuthError> {
        if let Some(email) = self.cached_identity(token) {
            return Ok(email);
        }
        let introspection = self
            .introspection
            .as_ref()
            .ok_or(GoogleAuthError::IntrospectionNotConfigured)?;

        let introspected = introspection
            .introspect(token)
            .await
            .map_err(GoogleAuthError::IntrospectionUnavailable)?
            .ok_or(GoogleAuthError::Rejected)?;

        let email = verified_email(introspected.email, introspected.email_verified)?;
        self.cache_identity(token, &email, introspected.expires_in);
        Ok(email)
    }

    fn cached_identity(&self, token: &str) -> Option<String> {
        let now = self.clock.now_unix();
        let cache = self
            .introspection_cache
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        cache
            .get(token)
            .filter(|entry| entry.expires_at_unix > now)
            .map(|entry| entry.email.clone())
    }

    fn cache_identity(&self, token: &str, email: &str, expires_in: u64) {
        let ttl = expires_in.min(self.introspection_ttl_ceiling.as_secs());
        if ttl == 0 {
            return;
        }
        let expires_at_unix = self.clock.now_unix().saturating_add(ttl);
        let mut cache = self
            .introspection_cache
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // Bound the map by dropping whatever has already lapsed; the cache is
        // keyed by credential, so it must not accumulate one entry per token
        // ever presented.
        let now = self.clock.now_unix();
        cache.retain(|_, entry| entry.expires_at_unix > now);
        cache.insert(
            token.to_string(),
            CachedIdentity {
                email: email.to_string(),
                expires_at_unix,
            },
        );
    }

    /// The shared tail both Google paths converge on: a verified identity is
    /// looked up in the registry, and authentication succeeding is not
    /// authorization succeeding.
    fn resolve(&self, identity: &str) -> Result<RoleMapPrincipal, GoogleAuthError> {
        self.registry
            .lookup(identity)
            .map(RoleMapPrincipal::Token)
            .ok_or(GoogleAuthError::NotInRegistry)
    }

    /// Authenticate a presented credential, returning the typed reason on
    /// failure. [`AsyncVerifier::authenticate_async`] is this, with the reason
    /// narrowed to an [`AuthError`].
    pub async fn authenticate_credential(
        &self,
        token: &str,
    ) -> Result<RoleMapPrincipal, GoogleAuthError> {
        match classify(token) {
            Credential::GoogleIdToken => {
                let email = self.verify_id_token(token).await?;
                self.resolve(&email)
            }
            Credential::Opaque => {
                // Registry first: a pre-shared secret is a local map hit and
                // must not acquire a network round trip.
                if let Some(claims) = self.registry.lookup(token) {
                    return Ok(RoleMapPrincipal::Token(claims));
                }
                let email = self.introspect_access_token(token).await?;
                self.resolve(&email)
            }
        }
    }
}

/// Only the claims this design reads. Google sends more; ignoring them is
/// deliberate — every claim consumed becomes contract.
#[derive(Debug, Deserialize)]
struct GoogleIdClaims {
    #[serde(default)]
    email: Option<String>,
    #[serde(default, deserialize_with = "lenient_bool")]
    email_verified: bool,
}

fn verified_email(email: Option<String>, verified: bool) -> Result<String, GoogleAuthError> {
    let email = email
        .filter(|e| !e.trim().is_empty())
        .ok_or(GoogleAuthError::EmailMissing)?;
    if !verified {
        return Err(GoogleAuthError::EmailUnverified);
    }
    Ok(email)
}

#[async_trait]
impl AsyncVerifier for GoogleVerifier {
    type Principal = RoleMapPrincipal;

    async fn authenticate_async(&self, headers: &HeaderMap) -> Result<RoleMapPrincipal, AuthError> {
        match (self.required, bearer_token(headers)) {
            (false, None) => Ok(RoleMapPrincipal::Open),
            (_, Some(token)) => self.authenticate_credential(token).await.map_err(|error| {
                if error.is_upstream_failure() {
                    tracing::warn!(
                        target: "service_auth.audit",
                        event = "identity_provider_unavailable",
                        reason = %error,
                    );
                }
                AuthError::from(error)
            }),
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
    use crate::role_map::{Role, TokenClaims};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

    /// A throwaway RSA key generated for this test suite alone. It signs
    /// nothing real; the matching JWKS below is what makes offline
    /// verification assertable without a live Google token.
    const SIGNING_KEY: &str = include_str!("../fixtures/gcp/throwaway-signing-key.pem");
    const JWKS: &str = include_str!("../fixtures/gcp/throwaway-jwks.json");
    const KID: &str = "test-key-1";
    const AUDIENCE: &str = "lumen";
    const IDENTITY: &str = "lumen-dev@axiom-502607.iam.gserviceaccount.com";

    // -- fakes ------------------------------------------------------------

    struct FakeClock(AtomicU64);
    impl FakeClock {
        fn new(at: u64) -> Arc<Self> {
            Arc::new(Self(AtomicU64::new(at)))
        }
        fn advance(&self, secs: u64) {
            self.0.fetch_add(secs, Ordering::SeqCst);
        }
    }
    impl Clock for FakeClock {
        fn now_unix(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    struct CountingJwks {
        calls: AtomicUsize,
        body: Option<String>,
    }
    impl CountingJwks {
        fn serving() -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                body: Some(JWKS.to_string()),
            })
        }
        fn unavailable() -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                body: None,
            })
        }
        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }
    #[async_trait]
    impl JwksSource for CountingJwks {
        async fn fetch(&self) -> Result<JwkSet, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match &self.body {
                Some(body) => serde_json::from_str(body).map_err(|e| e.to_string()),
                None => Err("connection refused".to_string()),
            }
        }
    }

    enum IntrospectionBehaviour {
        Live { email: String, expires_in: u64 },
        Invalid,
        Unreachable,
    }
    struct CountingIntrospection {
        calls: AtomicUsize,
        behaviour: IntrospectionBehaviour,
    }
    impl CountingIntrospection {
        fn live(email: &str, expires_in: u64) -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                behaviour: IntrospectionBehaviour::Live {
                    email: email.to_string(),
                    expires_in,
                },
            })
        }
        fn invalid() -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                behaviour: IntrospectionBehaviour::Invalid,
            })
        }
        fn unreachable() -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                behaviour: IntrospectionBehaviour::Unreachable,
            })
        }
        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }
    #[async_trait]
    impl AccessTokenIntrospection for CountingIntrospection {
        async fn introspect(&self, _token: &str) -> Result<Option<IntrospectedToken>, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match &self.behaviour {
                IntrospectionBehaviour::Live { email, expires_in } => Ok(Some(IntrospectedToken {
                    email: Some(email.clone()),
                    email_verified: true,
                    expires_in: *expires_in,
                })),
                IntrospectionBehaviour::Invalid => Ok(None),
                IntrospectionBehaviour::Unreachable => Err("connection refused".to_string()),
            }
        }
    }

    // -- helpers ----------------------------------------------------------

    fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn mint_with_kid(kid: &str, claims: serde_json::Value) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_string());
        encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(SIGNING_KEY.as_bytes()).unwrap(),
        )
        .unwrap()
    }

    fn id_token() -> String {
        mint_with_kid(
            KID,
            serde_json::json!({
                "iss": "https://accounts.google.com",
                "aud": AUDIENCE,
                "sub": "1234567890",
                "email": IDENTITY,
                "email_verified": true,
                "exp": now_unix() + 3600,
            }),
        )
    }

    /// The registry keyed by IAM identity instead of by secret. The key is a
    /// public email, so it can live in git, in a CR, in a code review — none
    /// of which is true of a bearer secret.
    fn registry() -> Arc<ReloadableRoleMapVerifier> {
        Arc::new(ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([(
                IDENTITY.to_string(),
                TokenClaims {
                    subject: "dev:lumen-dev".to_string(),
                    roles: HashMap::from([("products".to_string(), Role::Read)]),
                },
            )]),
        ))
    }

    fn verifier_with(
        jwks: Arc<dyn JwksSource>,
        introspection: Option<Arc<dyn AccessTokenIntrospection>>,
        clock: Arc<dyn Clock>,
    ) -> GoogleVerifier {
        GoogleVerifier::with_sources(
            true,
            registry(),
            GoogleAuthConfig::new([AUDIENCE]),
            jwks,
            introspection,
            clock,
        )
    }

    fn offline_verifier() -> GoogleVerifier {
        verifier_with(CountingJwks::serving(), None, FakeClock::new(1_700_000_000))
    }

    // -- AC1: the positive offline path -----------------------------------

    #[tokio::test]
    async fn real_shaped_id_token_verifies_offline_against_the_published_jwks() {
        let verifier = offline_verifier();
        let email = verifier.verify_id_token(&id_token()).await.unwrap();
        assert_eq!(email, IDENTITY);
    }

    #[tokio::test]
    async fn steady_state_verification_makes_no_network_call() {
        let jwks = CountingJwks::serving();
        let verifier = verifier_with(jwks.clone(), None, FakeClock::new(1_700_000_000));
        let token = id_token();

        verifier.verify_id_token(&token).await.unwrap();
        assert_eq!(jwks.calls(), 1, "the first verification primes the cache");
        for _ in 0..5 {
            verifier.verify_id_token(&token).await.unwrap();
        }
        assert_eq!(jwks.calls(), 1, "a warm cache must not touch the network");
    }

    // -- AC8 (from the spike): the identity drives the UNMODIFIED role map --

    #[tokio::test]
    async fn verified_identity_drives_the_unmodified_role_map() {
        let verifier = offline_verifier();
        let principal = verifier.authenticate_credential(&id_token()).await.unwrap();

        assert!(
            principal.ensure("products", Role::Read).is_ok(),
            "granted collection at the granted role -> allowed"
        );
        assert!(
            principal.ensure("products", Role::Admin).is_err(),
            "granted collection above the granted role -> denied"
        );
        assert!(
            principal.ensure("secrets", Role::Read).is_err(),
            "ungranted collection -> denied"
        );
        assert_eq!(
            principal.subject(),
            Some("dev:lumen-dev"),
            "audit subject survives the hop"
        );
    }

    // -- AC2: each negative rejected, and distinguishable -------------------

    #[tokio::test]
    async fn token_minted_for_another_audience_is_rejected_as_audience() {
        let verifier = offline_verifier();
        let token = mint_with_kid(
            KID,
            serde_json::json!({
                "iss": "https://accounts.google.com",
                "aud": "some-other-service",
                "email": IDENTITY, "email_verified": true,
                "exp": now_unix() + 3600,
            }),
        );
        assert_eq!(
            verifier.verify_id_token(&token).await.unwrap_err(),
            GoogleAuthError::Invalid(InvalidReason::Audience)
        );
    }

    #[tokio::test]
    async fn token_from_an_untrusted_issuer_is_rejected_as_issuer() {
        let verifier = offline_verifier();
        let token = mint_with_kid(
            KID,
            serde_json::json!({
                "iss": "https://evil.example.com",
                "aud": AUDIENCE,
                "email": IDENTITY, "email_verified": true,
                "exp": now_unix() + 3600,
            }),
        );
        assert_eq!(
            verifier.verify_id_token(&token).await.unwrap_err(),
            GoogleAuthError::Invalid(InvalidReason::Issuer)
        );
    }

    #[tokio::test]
    async fn tampered_signature_is_rejected_as_signature() {
        let verifier = offline_verifier();
        let token = id_token();
        // Flip one character of the signature; everything else stays
        // byte-identical.
        let (body, signature) = token.rsplit_once('.').unwrap();
        let first = signature.chars().next().unwrap();
        let flipped = if first == 'A' { 'B' } else { 'A' };
        let tampered = format!("{body}.{flipped}{}", &signature[1..]);

        assert_eq!(
            verifier.verify_id_token(&tampered).await.unwrap_err(),
            GoogleAuthError::Invalid(InvalidReason::Signature)
        );
    }

    #[tokio::test]
    async fn expired_token_is_rejected_as_expired() {
        let verifier = offline_verifier();
        let token = mint_with_kid(
            KID,
            serde_json::json!({
                "iss": "https://accounts.google.com",
                "aud": AUDIENCE,
                "email": IDENTITY, "email_verified": true,
                "exp": now_unix() - 3600,
            }),
        );
        assert_eq!(
            verifier.verify_id_token(&token).await.unwrap_err(),
            GoogleAuthError::Invalid(InvalidReason::Expired)
        );
    }

    #[tokio::test]
    async fn unverified_email_is_rejected_distinctly_from_a_missing_one() {
        let verifier = offline_verifier();
        let unverified = mint_with_kid(
            KID,
            serde_json::json!({
                "iss": "https://accounts.google.com", "aud": AUDIENCE,
                "email": IDENTITY, "email_verified": false,
                "exp": now_unix() + 3600,
            }),
        );
        assert_eq!(
            verifier.verify_id_token(&unverified).await.unwrap_err(),
            GoogleAuthError::EmailUnverified
        );

        let no_email = mint_with_kid(
            KID,
            serde_json::json!({
                "iss": "https://accounts.google.com", "aud": AUDIENCE,
                "exp": now_unix() + 3600,
            }),
        );
        assert_eq!(
            verifier.verify_id_token(&no_email).await.unwrap_err(),
            GoogleAuthError::EmailMissing
        );
    }

    // -- AC3: kid miss refetches exactly once per window --------------------

    fn token_with_unknown_kid() -> String {
        mint_with_kid(
            "rotated-key-2",
            serde_json::json!({
                "iss": "https://accounts.google.com", "aud": AUDIENCE,
                "email": IDENTITY, "email_verified": true,
                "exp": now_unix() + 3600,
            }),
        )
    }

    fn unknown_kid_error() -> GoogleAuthError {
        GoogleAuthError::UnknownSigningKey {
            kid: "rotated-key-2".to_string(),
        }
    }

    #[tokio::test]
    async fn unknown_kid_refetches_once_then_is_rate_limited() {
        let jwks = CountingJwks::serving();
        let clock = FakeClock::new(1_700_000_000);
        let verifier = verifier_with(jwks.clone(), None, clock.clone());
        let rotated = token_with_unknown_kid();

        assert_eq!(
            verifier.verify_id_token(&rotated).await.unwrap_err(),
            unknown_kid_error()
        );
        assert_eq!(jwks.calls(), 1, "a kid miss triggers exactly one fetch");

        assert_eq!(
            verifier.verify_id_token(&rotated).await.unwrap_err(),
            unknown_kid_error()
        );
        assert_eq!(
            jwks.calls(),
            1,
            "a second miss inside the window must not reach Google — this is \
             what keeps fabricated kid values from being amplified upstream"
        );

        clock.advance(DEFAULT_JWKS_REFETCH_MIN_INTERVAL.as_secs() + 1);
        let _ = verifier.verify_id_token(&rotated).await;
        assert_eq!(jwks.calls(), 2, "the window reopens after the interval");
    }

    #[tokio::test]
    async fn a_key_set_fetched_inside_the_window_is_treated_as_authoritative() {
        let jwks = CountingJwks::serving();
        let clock = FakeClock::new(1_700_000_000);
        let verifier = verifier_with(jwks.clone(), None, clock.clone());

        verifier.verify_id_token(&id_token()).await.unwrap();
        assert_eq!(jwks.calls(), 1);

        // A kid missing from a key set fetched moments ago is missing because
        // Google does not publish it, not because the cache is stale.
        // Refetching would ask the same question and get the same answer.
        assert_eq!(
            verifier
                .verify_id_token(&token_with_unknown_kid())
                .await
                .unwrap_err(),
            unknown_kid_error()
        );
        assert_eq!(jwks.calls(), 1);

        // The cost of that rule, pinned so it stays a decision rather than a
        // surprise: a genuine rotation is invisible until the window reopens.
        clock.advance(DEFAULT_JWKS_REFETCH_MIN_INTERVAL.as_secs() + 1);
        let _ = verifier.verify_id_token(&token_with_unknown_kid()).await;
        assert_eq!(jwks.calls(), 2);
    }

    #[tokio::test]
    async fn unreachable_jwks_is_an_upstream_failure_not_a_rejection() {
        let verifier = verifier_with(
            CountingJwks::unavailable(),
            None,
            FakeClock::new(1_700_000_000),
        );
        let error = verifier.verify_id_token(&id_token()).await.unwrap_err();
        assert!(matches!(error, GoogleAuthError::SigningKeyUnavailable(_)));
        assert!(error.is_upstream_failure());
        assert!(matches!(AuthError::from(error), AuthError::Unavailable(_)));
    }

    // -- AC4: introspection is cached --------------------------------------

    #[tokio::test]
    async fn introspection_result_is_cached_for_the_bounded_ttl() {
        let introspection = CountingIntrospection::live(IDENTITY, 3147);
        let clock = FakeClock::new(1_700_000_000);
        let verifier = verifier_with(
            CountingJwks::serving(),
            Some(introspection.clone()),
            clock.clone(),
        );

        for _ in 0..4 {
            let email = verifier
                .introspect_access_token("ya29.opaque")
                .await
                .unwrap();
            assert_eq!(email, IDENTITY);
        }
        assert_eq!(
            introspection.calls(),
            1,
            "repeats inside the TTL must not re-ask Google"
        );

        // expires_in is 3147s but the ceiling is 300s, so the ceiling wins.
        clock.advance(DEFAULT_INTROSPECTION_TTL_CEILING.as_secs() + 1);
        verifier
            .introspect_access_token("ya29.opaque")
            .await
            .unwrap();
        assert_eq!(
            introspection.calls(),
            2,
            "the ceiling, not expires_in, bounds how long a revoked credential works"
        );
    }

    // -- AC5: unreachable is distinct from rejected ------------------------

    #[tokio::test]
    async fn unreachable_introspection_is_distinct_from_a_rejected_credential() {
        let unreachable = verifier_with(
            CountingJwks::serving(),
            Some(CountingIntrospection::unreachable()),
            FakeClock::new(1_700_000_000),
        );
        let outage = unreachable
            .introspect_access_token("ya29.opaque")
            .await
            .unwrap_err();

        let rejecting = verifier_with(
            CountingJwks::serving(),
            Some(CountingIntrospection::invalid()),
            FakeClock::new(1_700_000_000),
        );
        let rejected = rejecting
            .introspect_access_token("ya29.opaque")
            .await
            .unwrap_err();

        assert!(matches!(
            outage,
            GoogleAuthError::IntrospectionUnavailable(_)
        ));
        assert_eq!(rejected, GoogleAuthError::Rejected);
        assert_ne!(outage, rejected);

        // And they must not collapse into the same thing on the wire either.
        assert!(matches!(AuthError::from(outage), AuthError::Unavailable(_)));
        assert!(matches!(
            AuthError::from(rejected),
            AuthError::Unauthenticated
        ));
    }

    // -- AC6: authentication is not authorization --------------------------

    #[tokio::test]
    async fn verified_identity_absent_from_the_registry_is_rejected() {
        let verifier = verifier_with(
            CountingJwks::serving(),
            Some(CountingIntrospection::live(
                "someone-else@axiom-502607.iam.gserviceaccount.com",
                3600,
            )),
            FakeClock::new(1_700_000_000),
        );
        let error = verifier
            .authenticate_credential("ya29.stranger")
            .await
            .unwrap_err();
        assert_eq!(error, GoogleAuthError::NotInRegistry);
        assert!(!error.is_upstream_failure());
    }

    // -- R5: discrimination by shape, not by trying each in turn ------------

    #[test]
    fn credentials_are_classified_by_shape() {
        assert_eq!(classify(&id_token()), Credential::GoogleIdToken);
        assert_eq!(classify("ya29.a0AfB_opaque"), Credential::Opaque);
        assert_eq!(classify("plain-preshared-secret"), Credential::Opaque);
        assert_eq!(classify("not.a.jwt"), Credential::Opaque);
        // Three segments but no kid: Google always sets one, so this is not a
        // Google ID token and must not be sent down the offline path.
        let mut header = Header::new(Algorithm::RS256);
        header.kid = None;
        let kidless = encode(
            &header,
            &serde_json::json!({"sub": "x", "exp": now_unix() + 60}),
            &EncodingKey::from_rsa_pem(SIGNING_KEY.as_bytes()).unwrap(),
        )
        .unwrap();
        assert_eq!(classify(&kidless), Credential::Opaque);
    }

    #[tokio::test]
    async fn a_preshared_secret_resolves_from_the_registry_without_asking_google() {
        let introspection = CountingIntrospection::live(IDENTITY, 3600);
        let registry = Arc::new(ReloadableRoleMapVerifier::new(
            true,
            HashMap::from([(
                "preshared-secret".to_string(),
                TokenClaims {
                    subject: "svc:legacy".to_string(),
                    roles: HashMap::from([("products".to_string(), Role::Write)]),
                },
            )]),
        ));
        let verifier = GoogleVerifier::with_sources(
            true,
            registry,
            GoogleAuthConfig::new([AUDIENCE]),
            CountingJwks::serving(),
            Some(introspection.clone()),
            FakeClock::new(1_700_000_000),
        );

        let principal = verifier
            .authenticate_credential("preshared-secret")
            .await
            .unwrap();
        assert_eq!(principal.subject(), Some("svc:legacy"));
        assert_eq!(
            introspection.calls(),
            0,
            "an existing bearer secret must not acquire a Google round trip"
        );
    }

    #[tokio::test]
    async fn opaque_credential_with_no_introspector_configured_says_so() {
        let verifier = verifier_with(CountingJwks::serving(), None, FakeClock::new(1_700_000_000));
        assert_eq!(
            verifier
                .authenticate_credential("ya29.opaque")
                .await
                .unwrap_err(),
            GoogleAuthError::IntrospectionNotConfigured
        );
    }

    // -- the middleware-facing surface -------------------------------------

    #[tokio::test]
    async fn authenticate_async_accepts_a_bearer_id_token() {
        let verifier = offline_verifier();
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", id_token()).parse().unwrap(),
        );
        let principal = verifier.authenticate_async(&headers).await.unwrap();
        assert_eq!(principal.subject(), Some("dev:lumen-dev"));
    }

    #[tokio::test]
    async fn authenticate_async_without_a_bearer_is_unauthenticated_when_required() {
        let verifier = offline_verifier();
        assert!(matches!(
            verifier.authenticate_async(&HeaderMap::new()).await,
            Err(AuthError::Unauthenticated)
        ));
    }

    // -- Google's string-typed JSON ----------------------------------------

    #[test]
    fn tokeninfo_string_typed_fields_are_understood() {
        // Measured against the live endpoint during design: Google sends
        // email_verified and expires_in as strings, not as JSON scalars.
        let parsed: IntrospectedToken = serde_json::from_str(
            r#"{"email":"a@b.com","email_verified":"true","expires_in":"3147"}"#,
        )
        .unwrap();
        assert_eq!(parsed.email.as_deref(), Some("a@b.com"));
        assert!(parsed.email_verified);
        assert_eq!(parsed.expires_in, 3147);

        let native: IntrospectedToken =
            serde_json::from_str(r#"{"email":"a@b.com","email_verified":true,"expires_in":3147}"#)
                .unwrap();
        assert!(native.email_verified);
        assert_eq!(native.expires_in, 3147);
    }

    #[test]
    fn debug_output_never_carries_a_credential() {
        let verifier = offline_verifier();
        let rendered = format!("{verifier:?}");
        assert!(!rendered.contains("ya29"));
        assert!(!rendered.contains("cache"));
    }
}
// HANDWRITE-END
