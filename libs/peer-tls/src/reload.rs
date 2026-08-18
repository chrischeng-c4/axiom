// HANDWRITE-BEGIN gap="missing-generator:logic:peer-tls-atomic-reload" tracker="#3112" reason="Last-known-good state, trust overlap, and fail-closed expiry are stateful policy over a lock; no generator primitive produces it."
//! Activating renewed TLS material in a process that is already serving.
//!
//! #3112 R2/R4/R5/R6/R7. Projecting a renewed Secret does not make a running
//! process use it, and the naive fixes are both wrong: restarting every member
//! for each short-lived leaf spends the disruption budget of a three-voter group
//! on routine renewal, and swapping the config in place risks a window where the
//! new material is half-installed.
//!
//! So the shape here is build-then-swap. A candidate is read, fully validated
//! (see [`crate::material`]), and turned into finished rustls configs *before*
//! any lock is taken for writing; the swap itself is a pointer move. Every
//! failure path — unreadable file, malformed PEM, wrong key, wrong identity —
//! leaves the previously activated generation exactly as it was, still serving.
//!
//! ### Two things this deliberately does not do
//!
//! It does not touch the listener. New handshakes read the current config when
//! they are accepted, so activation is atomic per connection and connections
//! already accepted finish on the configuration they started with, inside the
//! ordinary drain window.
//!
//! It does not fail open. If the last known good material expires and nothing
//! valid has replaced it, the accessors return `None` and the caller refuses the
//! connection. An expired identity that keeps serving is worse than a refused
//! connection, because nothing downstream can tell it apart from a healthy one.

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use rustls::pki_types::CertificateDer;
use rustls::{ClientConfig, RootCertStore, ServerConfig};

use crate::material::{
    validate, IdentityExpectation, MaterialPem, Rejection, RejectionReason, ValidatedMaterial,
};

/// Where a reloadable runtime reads its material from.
///
/// A trait rather than a path pair because the interesting deployments differ:
/// a mounted Secret is three files, an operator-driven runtime may hold the PEM
/// in memory, and a test wants neither. Everything downstream sees bytes.
pub trait MaterialSource: Send + Sync + 'static {
    fn load(&self) -> Result<MaterialPem, Rejection>;
}

/// The mounted-Secret case: `tls.crt`, `tls.key`, `ca.crt`.
#[derive(Debug, Clone)]
pub struct FileMaterialSource {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub trust_bundle: PathBuf,
}

impl FileMaterialSource {
    pub fn new(
        cert: impl Into<PathBuf>,
        key: impl Into<PathBuf>,
        trust_bundle: impl Into<PathBuf>,
    ) -> Self {
        Self {
            cert: cert.into(),
            key: key.into(),
            trust_bundle: trust_bundle.into(),
        }
    }

    /// The three files, in the order the reloader reads them. Callers watching
    /// for changes should watch exactly these.
    pub fn paths(&self) -> [&Path; 3] {
        [&self.cert, &self.key, &self.trust_bundle]
    }
}

impl MaterialSource for FileMaterialSource {
    fn load(&self) -> Result<MaterialPem, Rejection> {
        // The path is not in the error. A projection error reaches request logs
        // and status conditions, and a private-key path is a hint nobody outside
        // the process needs (R6). The `what` is enough to act on.
        let read = |path: &Path, what: &str| {
            std::fs::read(path).map_err(|err| {
                Rejection::new(
                    RejectionReason::Unreadable,
                    format!("{what} could not be read: {}", err.kind()),
                )
            })
        };
        Ok(MaterialPem {
            cert_chain: read(&self.cert, "certificate")?,
            key: read(&self.key, "private key")?,
            trust_bundle: read(&self.trust_bundle, "trust bundle")?,
        })
    }
}

/// An in-memory source, for callers driven by a watch rather than a mount.
pub struct MemoryMaterialSource(RwLock<Option<MaterialPem>>);

impl MemoryMaterialSource {
    pub fn new(pem: MaterialPem) -> Self {
        Self(RwLock::new(Some(pem)))
    }

    pub fn empty() -> Self {
        Self(RwLock::new(None))
    }

    /// Replace what the next reload will see.
    pub fn set(&self, pem: MaterialPem) {
        *self.0.write().unwrap_or_else(|e| e.into_inner()) = Some(pem);
    }

    /// Make the next reload fail the way a missing mount does.
    pub fn clear(&self) {
        *self.0.write().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

impl MaterialSource for MemoryMaterialSource {
    fn load(&self) -> Result<MaterialPem, Rejection> {
        self.0
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or_else(|| Rejection::new(RejectionReason::Unreadable, "no material available"))
    }
}

/// How a runtime's configs are built, and what its leaf must prove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsRuntimeProfile {
    pub identity: IdentityExpectation,
    pub alpn_protocols: Vec<Vec<u8>>,
    /// Require and verify a client certificate on the serving side.
    pub mutual: bool,
}

impl TlsRuntimeProfile {
    /// An ordinary public listener: ALPN offers `h2` and `http/1.1`, and clients
    /// are authenticated by their bearer token rather than by a certificate.
    ///
    /// Serving TLS proves *the server* and encrypts the token in flight; it is
    /// not an authorization input, and adding client-certificate requirements
    /// here would quietly turn it into one (#3112 R8).
    pub fn serving(dns_names: impl IntoIterator<Item = String>) -> Self {
        Self {
            identity: IdentityExpectation::serving(dns_names),
            alpn_protocols: vec![b"h2".to_vec(), b"http/1.1".to_vec()],
            mutual: false,
        }
    }

    /// A peer/replication port: mutual TLS, `h2` only.
    pub fn peer(
        dns_names: impl IntoIterator<Item = String>,
        spiffe_uris: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            identity: IdentityExpectation::peer(dns_names, spiffe_uris),
            alpn_protocols: vec![b"h2".to_vec()],
            mutual: true,
        }
    }
}

/// One activated generation.
struct Activation {
    generation: u64,
    material: ValidatedMaterial,
    server: Arc<ServerConfig>,
    client: Arc<ClientConfig>,
    trust_anchors: usize,
}

struct State {
    active: Option<Activation>,
    /// Anchors from the generation before the active one, kept trusted until the
    /// certificate controller confirms the fleet activated the new leaf (R5).
    retiring: Vec<CertificateDer<'static>>,
    accepted: u64,
    rejected: u64,
    last_error: Option<Rejection>,
}

/// The bounded status surface: generation, fingerprint, expiry, counters, and
/// the last refusal.
///
/// Everything here is a number, a stable enum spelling, or a fingerprint. There
/// is no field a PEM body or a filesystem path can travel in, which is the point
/// — this is what reaches metrics and health, and those are read by things that
/// are not allowed to learn key material (#3112 R6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsReloadStatus {
    /// Increments once per successful activation; `0` means nothing ever
    /// activated.
    pub generation: u64,
    /// Lowercase hex sha256 of the active leaf, in the certificate controller's
    /// own spelling, so "did the runtime pick it up" is a string comparison.
    pub fingerprint: Option<String>,
    /// Seconds until the active leaf expires; `Some(0)` means expired.
    pub seconds_to_expiry: Option<u64>,
    pub accepted_reloads: u64,
    pub rejected_reloads: u64,
    /// Stable spelling of why the most recent refusal happened.
    pub last_error_reason: Option<&'static str>,
    /// Human-readable detail for the same refusal.
    pub last_error: Option<String>,
    pub trust_anchors: usize,
    /// Anchors retained from the previous generation, still accepted.
    pub retiring_trust_anchors: usize,
    /// Whether a handshake attempted right now would get a configuration.
    pub serving: bool,
}

/// TLS material that can be replaced while the process keeps serving.
#[derive(Clone)]
pub struct ReloadableTls {
    inner: Arc<Inner>,
}

struct Inner {
    profile: TlsRuntimeProfile,
    source: Arc<dyn MaterialSource>,
    state: RwLock<State>,
}

impl fmt::Debug for ReloadableTls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = self.status();
        f.debug_struct("ReloadableTls")
            .field("generation", &status.generation)
            .field("serving", &status.serving)
            .finish_non_exhaustive()
    }
}

impl ReloadableTls {
    /// Load and activate material now, refusing to exist without it.
    ///
    /// This is the production constructor and the whole of R7's startup half: a
    /// service that requires TLS and cannot prove an identity has nothing useful
    /// to do next, and starting anyway would publish a listener that either
    /// serves plaintext or fails every handshake while readiness says otherwise.
    pub fn required(
        profile: TlsRuntimeProfile,
        source: Arc<dyn MaterialSource>,
    ) -> Result<Self, Rejection> {
        Self::required_at(profile, source, SystemTime::now())
    }

    /// [`Self::required`] against an explicit instant, for deterministic tests.
    pub fn required_at(
        profile: TlsRuntimeProfile,
        source: Arc<dyn MaterialSource>,
        now: SystemTime,
    ) -> Result<Self, Rejection> {
        let tls = Self::pending(profile, source);
        tls.reload_at(now)?;
        Ok(tls)
    }

    /// Construct without material, for a runtime that expects its first
    /// projection to arrive later. Nothing is served until a reload succeeds.
    pub fn pending(profile: TlsRuntimeProfile, source: Arc<dyn MaterialSource>) -> Self {
        Self {
            inner: Arc::new(Inner {
                profile,
                source,
                state: RwLock::new(State {
                    active: None,
                    retiring: Vec::new(),
                    accepted: 0,
                    rejected: 0,
                    last_error: None,
                }),
            }),
        }
    }

    pub fn profile(&self) -> &TlsRuntimeProfile {
        &self.inner.profile
    }

    /// Read, validate, and activate the current material.
    pub fn reload(&self) -> Result<u64, Rejection> {
        self.reload_at(SystemTime::now())
    }

    /// [`Self::reload`] against an explicit instant.
    pub fn reload_at(&self, now: SystemTime) -> Result<u64, Rejection> {
        match self.try_activate(now) {
            Ok(generation) => Ok(generation),
            Err(rejection) => {
                self.record_rejection(rejection.clone());
                Err(rejection)
            }
        }
    }

    /// Everything that can fail, done before the write lock is taken.
    fn try_activate(&self, now: SystemTime) -> Result<u64, Rejection> {
        let pem = self.inner.source.load()?;
        let material = validate(&pem, &self.inner.profile.identity, now)?;

        // The anchors the previous generation was validated against stay trusted
        // across the swap: during an issuer rotation the fleet does not turn
        // over in one instant, and dropping the old root the moment this member
        // has a new leaf is how a rotation becomes a partition (R5).
        let carried = {
            let state = self.read_state();
            let mut carried = state.retiring.clone();
            if let Some(active) = &state.active {
                if active.material.fingerprint() == material.fingerprint() {
                    // Nothing changed. Re-activating would churn the generation
                    // for every poll tick and make "generation" useless as a
                    // signal that the leaf moved.
                    return Ok(active.generation);
                }
                carried.extend(active.material.trust_anchors().iter().cloned());
            }
            carried
        };

        let (server, client, trust_anchors) =
            build_configs(&self.inner.profile, &material, &carried)?;

        let mut state = self.write_state();
        let generation = state
            .active
            .as_ref()
            .map(|active| active.generation)
            .unwrap_or(0)
            .saturating_add(1);
        let previous_anchors = state
            .active
            .as_ref()
            .map(|active| active.material.trust_anchors().to_vec())
            .unwrap_or_default();
        state.retiring = dedupe(state.retiring.drain(..).chain(previous_anchors));
        state.active = Some(Activation {
            generation,
            material,
            server,
            client,
            trust_anchors,
        });
        state.accepted = state.accepted.saturating_add(1);
        state.last_error = None;
        Ok(generation)
    }

    /// Drop the trust carried from earlier generations.
    ///
    /// Called once the certificate controller has observed every member running
    /// `generation`. Returns `false` — and changes nothing — when the argument
    /// names some other generation, because a retirement racing an activation
    /// would otherwise retire trust the fleet has not finished adopting (R5).
    pub fn retire_previous_trust(&self, generation: u64) -> bool {
        self.retire_previous_trust_at(generation, SystemTime::now())
    }

    /// [`Self::retire_previous_trust`] against an explicit instant.
    pub fn retire_previous_trust_at(&self, generation: u64, now: SystemTime) -> bool {
        let mut state = self.write_state();
        let Some(active) = &state.active else {
            return false;
        };
        if active.generation != generation {
            return false;
        }
        if state.retiring.is_empty() {
            return true;
        }
        let Ok((server, client, trust_anchors)) =
            build_configs(&self.inner.profile, &active.material, &[])
        else {
            return false;
        };
        let _ = now;
        state.retiring.clear();
        if let Some(active) = state.active.as_mut() {
            active.server = server;
            active.client = client;
            active.trust_anchors = trust_anchors;
        }
        true
    }

    /// The server configuration a handshake accepted right now should use, or
    /// `None` when nothing valid is active.
    pub fn server_config(&self) -> Option<Arc<ServerConfig>> {
        self.server_config_at(SystemTime::now())
    }

    /// [`Self::server_config`] against an explicit instant.
    pub fn server_config_at(&self, now: SystemTime) -> Option<Arc<ServerConfig>> {
        let state = self.read_state();
        let active = state.active.as_ref()?;
        active
            .material
            .is_valid_at(now)
            .then(|| Arc::clone(&active.server))
    }

    /// The client configuration for dialing a peer, or `None` when nothing valid
    /// is active.
    pub fn client_config(&self) -> Option<Arc<ClientConfig>> {
        self.client_config_at(SystemTime::now())
    }

    /// [`Self::client_config`] against an explicit instant.
    pub fn client_config_at(&self, now: SystemTime) -> Option<Arc<ClientConfig>> {
        let state = self.read_state();
        let active = state.active.as_ref()?;
        active
            .material
            .is_valid_at(now)
            .then(|| Arc::clone(&active.client))
    }

    /// The generation currently activated; `0` before the first activation.
    pub fn generation(&self) -> u64 {
        self.read_state()
            .active
            .as_ref()
            .map(|active| active.generation)
            .unwrap_or(0)
    }

    /// Fingerprint of the active leaf.
    pub fn fingerprint(&self) -> Option<String> {
        self.read_state()
            .active
            .as_ref()
            .map(|active| active.material.fingerprint().to_string())
    }

    pub fn status(&self) -> TlsReloadStatus {
        self.status_at(SystemTime::now())
    }

    /// [`Self::status`] against an explicit instant.
    pub fn status_at(&self, now: SystemTime) -> TlsReloadStatus {
        let state = self.read_state();
        let active = state.active.as_ref();
        TlsReloadStatus {
            generation: active.map(|a| a.generation).unwrap_or(0),
            fingerprint: active.map(|a| a.material.fingerprint().to_string()),
            seconds_to_expiry: active.map(|a| a.material.seconds_to_expiry(now)),
            accepted_reloads: state.accepted,
            rejected_reloads: state.rejected,
            last_error_reason: state.last_error.as_ref().map(|e| e.reason.as_str()),
            last_error: state.last_error.as_ref().map(|e| e.detail.clone()),
            trust_anchors: active.map(|a| a.trust_anchors).unwrap_or(0),
            retiring_trust_anchors: state.retiring.len(),
            serving: active.is_some_and(|a| a.material.is_valid_at(now)),
        }
    }

    fn record_rejection(&self, rejection: Rejection) {
        let mut state = self.write_state();
        state.rejected = state.rejected.saturating_add(1);
        state.last_error = Some(rejection);
    }

    fn read_state(&self) -> std::sync::RwLockReadGuard<'_, State> {
        self.inner
            .state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write_state(&self) -> std::sync::RwLockWriteGuard<'_, State> {
        self.inner
            .state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Poll cadence for a projected Secret. kubelet refreshes a projected volume on
/// its own schedule, so a short poll buys nothing; a leaf renewed hours before
/// expiry has no deadline this misses.
pub const DEFAULT_MATERIAL_POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Poll the source and activate whatever validates, forever.
///
/// A rejected candidate is counted and logged, never fatal: the material on disk
/// during a two-step Secret update is briefly inconsistent by construction, and
/// a watcher that gave up on the first bad read would turn every rotation into
/// an outage.
pub fn spawn_material_watcher(
    tls: ReloadableTls,
    interval: Duration,
) -> tokio::task::JoinHandle<()> {
    let interval = if interval.is_zero() {
        Duration::from_secs(1)
    } else {
        interval
    };
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            if let Err(rejection) = tls.reload() {
                tracing::warn!(
                    target: "peer_tls.reload",
                    reason = rejection.reason.as_str(),
                    detail = %rejection.detail,
                    "projected TLS material rejected; retaining the last valid generation"
                );
            }
        }
    })
}

fn build_configs(
    profile: &TlsRuntimeProfile,
    material: &ValidatedMaterial,
    carried: &[CertificateDer<'static>],
) -> Result<(Arc<ServerConfig>, Arc<ClientConfig>, usize), Rejection> {
    let anchors = dedupe(
        material
            .trust_anchors()
            .iter()
            .cloned()
            .chain(carried.iter().cloned()),
    );
    let mut roots = RootCertStore::empty();
    for anchor in &anchors {
        roots.add(anchor.clone()).map_err(|err| {
            Rejection::new(
                RejectionReason::MalformedPem,
                format!("trust anchor rejected: {err}"),
            )
        })?;
    }
    let roots = Arc::new(roots);

    let builder = ServerConfig::builder();
    let builder = if profile.mutual {
        let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::clone(&roots))
            .build()
            .map_err(|err| {
                Rejection::new(
                    RejectionReason::EmptyTrustBundle,
                    format!("client verifier: {err}"),
                )
            })?;
        builder.with_client_cert_verifier(verifier)
    } else {
        builder.with_no_client_auth()
    };
    let mut server = builder
        .with_single_cert(material.chain().to_vec(), material.key())
        .map_err(|err| Rejection::new(RejectionReason::KeyMismatch, err.to_string()))?;
    server.alpn_protocols = profile.alpn_protocols.clone();

    let mut client = ClientConfig::builder()
        .with_root_certificates((*roots).clone())
        .with_client_auth_cert(material.chain().to_vec(), material.key())
        .map_err(|err| Rejection::new(RejectionReason::KeyMismatch, err.to_string()))?;
    client.alpn_protocols = profile.alpn_protocols.clone();

    let count = anchors.len();
    Ok((Arc::new(server), Arc::new(client), count))
}

/// Preserve order, drop repeats. A bundle that already carries current+next and
/// a carried previous bundle overlap by construction; adding the same anchor
/// twice makes `RootCertStore::add` reject the second copy.
fn dedupe(
    anchors: impl IntoIterator<Item = CertificateDer<'static>>,
) -> Vec<CertificateDer<'static>> {
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    let mut out = Vec::new();
    for anchor in anchors {
        if seen.insert(anchor.as_ref().to_vec()) {
            out.push(anchor);
        }
    }
    out
}
// HANDWRITE-END
