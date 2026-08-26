//! axum HTTP application over the tape journal.
//!
//! `append` / `replay` / `checkpoint_get` / `checkpoint_put` are thin
//! handlers wrapping the unchanged [`crate::TapeJournal`] API — no new
//! domain behavior lives here. The operational surface is the shared
//! service shell: the standard probe routes (`/healthz` `/readyz`
//! `/metrics` `/openapi.json` `/docs`) come from
//! `service_http::standard_probe_routes` merged with the `/topics` data
//! plane; error responses render the shared `{error, message}` envelope
//! ([`service_http::ApiErr`]); per-op request metrics are recorded by
//! [`crate::metrics::track`] on the data plane.
//!
//! Request auth is the shared `libs/service-auth` bearer contract (#1326):
//! the blanket `service_auth::auth_middleware` runs on the `/topics` data
//! plane ONLY (probes stay tokenless), injecting an [`AuditedRoleMapPrincipal`] each
//! handler authorizes on its `{topic}` via [`crate::auth::authorize`] —
//! `append` = write, `replay`/`checkpoint_get`/`checkpoint_put` = read.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::{Extension, Path, Query, State};
use axum::http::{header, Method, StatusCode};
use axum::middleware::{from_fn, from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use service_auth::{AuditedRoleMapPrincipal, ReloadableRoleMapVerifier, Role};
use service_http::{ApiErr, MetricsProvider};
use utoipa::ToSchema;

use crate::metrics::TapeMetrics;
use crate::raft::{apply_command, TapeCommand, TapeOutcome, TapeRaft};
use crate::wal::CommitCoordinator;
use crate::{
    ConsumerCheckpoint, PullSubscriptionBatch, RetentionPolicy, Subscription, SubscriptionAckError,
    SubscriptionError, TapeError, TapeEvent, TapeJournal,
};
use metrics_prometheus::{escape_label_value, Label, LabeledSample, SampleGroup};

/// Which durable backend a single-node `AppState` mutates through. `Raft`
/// mode (`AppState::raft` is `Some`) bypasses this enum entirely -- it is
/// only consulted by [`AppState::apply_mutation`] on the non-replicated
/// path.
///
/// #3052: `Wal` is `tape serve --data-dir`'s new group-commit path;
/// `LegacyFile` is the unchanged whole-file JSON `--store <file>` path used
/// by both the CLI's offline verbs and any caller that still passes an
/// explicit `--store`; `None` matches today's "no journal store configured"
/// case (durability-free, e.g. tests or an ephemeral run).
///
/// Deliberately NOT plumbed through [`AppState::new`] / [`AppState::
/// with_auth`]'s signatures: those two constructors have dozens of existing
/// call sites across this crate's tests, and widening their `Option<PathBuf>`
/// parameter would force every one of them to learn about a WAL they don't
/// use. [`AppState::with_wal`] is the one additional seam `tape serve
/// --data-dir` opts into.
#[derive(Clone)]
enum Durability {
    Wal(Arc<CommitCoordinator>),
    // The path itself is never read back off this variant -- `persist` still
    // reads `AppState::store` directly -- so this only records which kind of
    // backend is configured. Kept as `PathBuf` (not unit) so a future reader
    // of `apply_mutation`'s match arms can see the path is the same one
    // `store` holds, without needing a second field name to reach for.
    #[allow(dead_code)]
    LegacyFile(PathBuf),
    None,
}

/// Shared application state: the journal (behind a `std::sync::Mutex` — an
/// in-memory `BTreeMap` core with no async internal awaits), the per-op
/// request metrics, the drain flag `/readyz` reports, the optional file the
/// journal persists to on every mutation (`--store`, mirroring the CLI's
/// `load_journal`/`save_journal`), the bearer verifier the data-plane auth
/// layer runs (#1326), the optional raft group (#1327) that replicates
/// append/checkpoint-put in HA (`REPLICAS_PER_SHARD > 1`) mode, and the
/// configured data-plane request body size limit (#2484). `raft` stays
/// `None` in single-node serving — the direct-journal path below is
/// unchanged.
#[derive(Clone)]
pub struct AppState {
    journal: Arc<Mutex<TapeJournal>>,
    metrics: Arc<TapeMetrics>,
    draining: Arc<AtomicBool>,
    store: Option<PathBuf>,
    /// #3052: which durable backend [`AppState::apply_mutation`] mutates
    /// through on the non-Raft path. Derived from `store` by every
    /// constructor (`LegacyFile`/`None`); [`AppState::with_wal`] is the only
    /// way to move a state into `Wal` mode.
    durability: Durability,
    verifier: Arc<ReloadableRoleMapVerifier>,
    raft: Option<Arc<TapeRaft>>,
    body_limit_bytes: usize,
    /// #2573: test-only ENOSPC fault injection, armed via
    /// [`AppState::set_inject_storage_full`]. Scoped to THIS state instance
    /// (not a process-global flag) so parallel `cargo test` threads sharing
    /// the same test binary never cross-contaminate — each test builds its own
    /// `AppState` over its own tempdir. `Arc` because `AppState` is `Clone` and
    /// the router hands a clone to every handler: arming the flag on the state
    /// a test holds has to be visible to the state the request runs against.
    #[cfg(test)]
    inject_storage_full: Arc<AtomicBool>,
}

impl AppState {
    /// Build state from an already-loaded journal (empty when no `--store`
    /// file exists yet, mirroring the CLI's `load_journal`). Auth is open
    /// (tokenless — the `TAPE_AUTH=off` default); production serving builds
    /// through [`AppState::with_auth`]. The `body_limit_bytes` parameter
    /// configures the data-plane request body size limit (#2484).
    pub fn new(journal: TapeJournal, store: Option<PathBuf>, body_limit_bytes: usize) -> Self {
        let durability = match &store {
            Some(path) => Durability::LegacyFile(path.clone()),
            None => Durability::None,
        };
        Self {
            journal: Arc::new(Mutex::new(journal)),
            metrics: Arc::new(TapeMetrics::new()),
            draining: Arc::new(AtomicBool::new(false)),
            store,
            durability,
            verifier: Arc::new(ReloadableRoleMapVerifier::open()),
            raft: None,
            body_limit_bytes,
            #[cfg(test)]
            inject_storage_full: Arc::new(AtomicBool::new(false)),
        }
    }

    /// #3052: move this state onto the WAL group-commit path for `tape serve
    /// --data-dir`. Deliberately a post-construction step (not a
    /// constructor parameter) so `AppState::new`/`AppState::with_auth`'s
    /// existing call sites -- dozens of them across this crate's tests --
    /// never need to learn about `CommitCoordinator`.
    ///
    /// Consuming and returning `self` matches `tape.rs`'s `serve_main`
    /// builder-style setup (`with_auth(..).with_wal(..)`-shaped call chain).
    pub fn with_wal(mut self, coordinator: Arc<CommitCoordinator>) -> Self {
        self.durability = Durability::Wal(coordinator);
        self
    }

    /// #2573: arm/disarm the next [`AppState::persist`] call on THIS state to
    /// fail with a synthetic `io::ErrorKind::StorageFull` instead of touching
    /// the real file — the fault-injection seam that exercises the REAL
    /// production path (`persist` -> [`TapeMetrics::mark_storage_degraded`] ->
    /// [`enforce_storage_writable`] -> the `507`/`storage_full` envelope) end
    /// to end without needing a genuinely full disk.
    ///
    /// A degraded mode that cannot be exercised in CI is one that will be
    /// wrong the first time it runs for real, so the seam is deliberate rather
    /// than a testing convenience. It is `#[cfg(test)]`, so it does not exist
    /// in a release binary and cannot be reached from a request.
    #[cfg(test)]
    pub fn set_inject_storage_full(&self, on: bool) {
        self.inject_storage_full.store(on, Ordering::SeqCst);
    }

    /// Build state with a resolved auth config (`--auth` /
    /// `--token-registry-file`): the data-plane auth layer runs the registry
    /// verifier when auth is required, the open verifier when off.
    pub fn with_auth(
        journal: TapeJournal,
        store: Option<PathBuf>,
        auth: crate::auth::AuthConfig,
        body_limit_bytes: usize,
    ) -> Self {
        let mut state = Self::new(journal, store, body_limit_bytes);
        state.verifier = Arc::new(auth.verifier());
        state
    }

    /// The bearer verifier the data-plane auth middleware runs.
    pub fn verifier(&self) -> Arc<ReloadableRoleMapVerifier> {
        Arc::clone(&self.verifier)
    }

    /// The per-op request metrics `/metrics` renders.
    pub fn metrics(&self) -> Arc<TapeMetrics> {
        Arc::clone(&self.metrics)
    }

    /// The shared journal handle, for wiring a [`crate::raft::TapeRaft`]
    /// group onto the SAME journal this state serves reads from (#1327).
    pub fn journal_handle(&self) -> Arc<Mutex<TapeJournal>> {
        Arc::clone(&self.journal)
    }

    /// Attach the raft group (auto-mode HA serve path, #1327). Once set,
    /// `append`/`checkpoint_put` propose through it instead of mutating the
    /// journal directly.
    pub fn set_raft(&mut self, raft: Arc<TapeRaft>) {
        self.raft = Some(raft);
    }

    /// The raft group this state proposes through, when running in HA mode.
    pub fn raft(&self) -> Option<Arc<TapeRaft>> {
        self.raft.clone()
    }

    /// The configured data-plane request body size limit (bytes).
    pub fn body_limit_bytes(&self) -> usize {
        self.body_limit_bytes
    }

    /// Flip readiness to draining so `/readyz` returns 503. Called on
    /// SIGTERM via `service_http::shutdown_with_drain`.
    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }

    /// Persist the journal to `--store`, when configured, mirroring the CLI's
    /// `save_journal`.
    ///
    /// Durability (#2572): the write goes through
    /// [`storage_durable::atomic_write`] — temp file, fsync, rename, parent
    /// directory fsync — so a crash, eviction, or ENOSPC mid-write leaves the
    /// *previous* journal intact rather than a truncated one. A plain
    /// `fs::write` truncates before writing and never fsyncs, which made a
    /// failed write destructive and a successful one unproven.
    ///
    /// [`FsyncPolicy::Always`] is deliberate: in single-node mode this file is
    /// tape's only durability guarantee (in replica/HA mode `store` is `None`
    /// and raft owns durability, so this path no-ops). The journal is
    /// re-serialized in full on every mutation, so the fsync is not the term
    /// that dominates this write.
    ///
    /// Failures are reported to the caller. #3052 moved the actual
    /// degraded-mode latch out of this function and into
    /// [`AppState::apply_mutation`], which now sits in front of every
    /// durable backend (this legacy whole-file path, and the WAL group-commit
    /// path); see that function's doc comment for the merged ENOSPC/EIO
    /// predicate. This function still preserves the failure's
    /// [`std::io::ErrorKind`] end to end (#2572) -- that is what makes the
    /// caller's discrimination possible at all.
    fn persist(&self, journal: &TapeJournal) -> std::io::Result<()> {
        let Some(path) = &self.store else {
            return Ok(());
        };
        let bytes = serde_json::to_vec_pretty(journal)?;
        let result =
            storage_durable::atomic_write(path, &bytes, storage_durable::FsyncPolicy::Always)
                .map_err(flatten_atomic_write_error);
        if let Err(error) = &result {
            // #2572 preserved the ErrorKind through `atomic_write`'s context
            // chain precisely so this discrimination is possible: a full disk
            // is a durable condition retrying cannot clear, every other I/O
            // failure may well be transient.
            if error.kind() == std::io::ErrorKind::StorageFull {
                tracing::error!(
                    error = %error,
                    path = %path.display(),
                    "journal persist hit ENOSPC; AppState::apply_mutation will latch \
                     degraded read-only mode"
                );
            }
        }
        result
    }

    /// Apply one mutating [`TapeCommand`] through whichever durable backend
    /// this state is configured with on the non-Raft path -- WAL group
    /// commit, the legacy whole-file `persist`, or no local store at all --
    /// and report the same [`TapeOutcome`] vocabulary
    /// [`crate::raft::apply_command`] produces for the Raft-replicated path,
    /// so every serving path (Raft propose, WAL commit, legacy persist)
    /// shares one mutation semantics.
    ///
    /// A domain-level rejection -- e.g. `TapeOutcome::Checkpoint(Err(TapeError
    /// ::StaleCheckpoint))` -- is NOT a durability failure and is reported as
    /// `Ok(TapeOutcome::Checkpoint(Err(..)))`, never this function's own
    /// `Err`. The command is applied (and, in WAL mode, already durably
    /// written) exactly as if it had succeeded -- mirroring the Raft path,
    /// whose `TapeStateMachine::apply` always returns `Ok(())` regardless of
    /// the wrapped `TapeOutcome`. Do not "validate before writing" to route
    /// around this asymmetry; that would silently re-diverge the single-node
    /// and Raft-replicated apply semantics #3052 unified. This function's own
    /// `Err` is reserved for durability failures only (fsync/rename/write
    /// errors on the legacy path; the coordinator's `Err` on the WAL path).
    async fn apply_mutation(&self, command: TapeCommand) -> std::io::Result<TapeOutcome> {
        // #2573 test seam (#3052: hoisted out of `persist` to here so it
        // fires identically ahead of EVERY backend, including WAL mode, not
        // only the legacy whole-file path `persist` used to gate alone). See
        // `AppState::set_inject_storage_full`.
        #[cfg(test)]
        if self.inject_storage_full.load(Ordering::SeqCst) {
            self.metrics.mark_storage_degraded();
            return Err(std::io::Error::from(std::io::ErrorKind::StorageFull));
        }

        let result = match &self.durability {
            Durability::Wal(coordinator) => coordinator.submit(command).await,
            Durability::LegacyFile(_) | Durability::None => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                let outcome = apply_command(&mut journal, command);
                self.persist(&journal).map(|()| outcome)
            }
        };

        if let Err(error) = &result {
            // WI #3052 R6: an ENOSPC OR EIO durability failure latches sticky
            // degraded mode -- narrowing this to ENOSPC alone would route a
            // durable EIO into a plain per-request failure, exactly the
            // "durability failure treated as an ordinary retryable request"
            // R6 forbids. Matches the accepted TD's
            // `should_enter_storage_degraded_mode` predicate. A failure that
            // is neither still fails this one request closed without
            // flipping the server into sticky read-only degraded mode.
            let is_enospc = error.kind() == std::io::ErrorKind::StorageFull;
            if is_enospc || is_eio(error) {
                self.metrics.mark_storage_degraded();
            }
        }

        result
    }
}

/// EIO on both platforms this crate builds for (Linux and macOS), named here
/// rather than pulled in via a `libc` dependency for one constant.
const EIO: i32 = 5;

/// Detect an underlying EIO that a durable write path reported.
///
/// This cannot be written as `error.kind() == ErrorKind::Other &&
/// error.raw_os_error() == Some(EIO)`, which is the obvious form and is dead
/// code in both halves: EIO maps to `ErrorKind::Uncategorized` (not `Other`,
/// and unnameable in stable Rust), and every rebuild through
/// `std::io::Error::new` -- which both the WAL flattener and the commit
/// coordinator's per-waiter fan-out must do -- erases `raw_os_error()`. The
/// errno is therefore carried explicitly; see [`crate::wal::DurabilityFailure`].
fn is_eio(error: &std::io::Error) -> bool {
    crate::wal::durability_errno(error) == Some(EIO)
}

/// #2573: sticky ENOSPC degraded read-only mode. Called by every mutating
/// handler right after it has authorized the caller, and before it touches the
/// journal — so a node that has already taken a genuine ENOSPC hit on its
/// persist path (see [`AppState::persist`]) fast-fails each subsequent
/// mutating request with `507 Insufficient Storage` and the machine-readable
/// `storage_full` code, instead of re-running (and re-failing) the same write.
/// A full disk then costs one status code per request, not one failed write.
///
/// Ordering is deliberate: authorization runs FIRST. Node storage state is
/// operational information, and an unauthenticated caller has no business
/// learning it — a caller who cannot write must see `401`/`403` whether the
/// disk is full or not.
///
/// Reads are exempt by construction (they never call this) and keep serving
/// while degraded, which is the whole point of "degraded read-only" rather
/// than "unready": a node that cannot accept appends can still answer every
/// replay and checkpoint read from the journal it already holds. `/readyz`
/// stays green for the same reason.
///
/// In replica/HA mode this is a no-op: `store` is `None`, so `persist`
/// no-ops, the gauge is never set, and raft-runtime owns the durable write
/// path (and any ENOSPC handling on it) instead.
fn enforce_storage_writable(state: &AppState) -> Result<(), ApiErr> {
    if state.metrics.is_storage_degraded() {
        return Err(ApiErr::new(
            StatusCode::INSUFFICIENT_STORAGE,
            "storage_full",
            "node is in degraded read-only mode: the journal store reported ENOSPC on its \
             durable write path; reads keep serving. Retry once the periodic re-probe \
             (TAPE_STORAGE_FULL_REPROBE_SECS, default 30s) clears it, or restart the pod \
             after freeing or expanding the volume",
        ));
    }
    Ok(())
}

/// #2573: render a failed [`AppState::persist`] as a response, discriminating
/// a full disk from every other I/O failure.
///
/// `507 Insufficient Storage` + `storage_full` for ENOSPC — the same envelope
/// [`enforce_storage_writable`] returns, so the request that FIRST hits a full
/// disk and every request after it are indistinguishable to a client. `500` +
/// `internal` for anything else, unchanged from before #2573.
fn persist_failure(error: std::io::Error) -> Response {
    if error.kind() == std::io::ErrorKind::StorageFull {
        return ApiErr::new(
            StatusCode::INSUFFICIENT_STORAGE,
            "storage_full",
            format!("journal persist failed: local storage is full (ENOSPC): {error}"),
        )
        .into_response();
    }
    ApiErr::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        error.to_string(),
    )
    .into_response()
}

/// Collapse `storage_durable::atomic_write`'s `anyhow::Error` back into an
/// `io::Error` without losing either half of it (#2572).
///
/// The **kind** is preserved by downcasting through the context chain, so a
/// full disk still reports [`std::io::ErrorKind::StorageFull`] rather than
/// `Other` — that is what lets a caller distinguish "the disk is full" from
/// "the write failed", which #2573's degraded read-only mode depends on.
///
/// The **message** uses anyhow's alternate form so the operator-facing 500
/// keeps the whole chain (`commit durable replace … -> …: No space left on
/// device`) instead of only the outermost context.
fn flatten_atomic_write_error(error: anyhow::Error) -> std::io::Error {
    match error.downcast_ref::<std::io::Error>() {
        Some(source) => std::io::Error::new(source.kind(), format!("{error:#}")),
        None => std::io::Error::other(format!("{error:#}")),
    }
}

/// Readiness source for the shared probe router: `/readyz` reports 503 once
/// SIGTERM flips `start_drain`.
impl service_http::ReadinessHook for AppState {
    fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

/// Prometheus exposition for the shared `/metrics` route: tape's per-op
/// request counts + latency, plus topic latest offset and subscription lag
/// gauges computed at scrape time under the journal mutex (#2485).
impl service_http::MetricsProvider for AppState {
    fn render_metrics(&self) -> String {
        let mut output = self.metrics.render();

        // Scrape-time computation of topic and subscription lag gauges under
        // the journal mutex. The journal is an in-memory BTreeMap; walking it
        // is O(topics + cursors) and consistent by construction. A very large
        // topic count would lengthen the scrape's lock hold — documented as
        // acceptable at tape's scale.
        let journal = match self.journal.lock() {
            Ok(j) => j,
            Err(_) => return output, // Poisoned mutex; return request metrics only
        };

        // Collect topic latest offsets and subscription lag samples. Build
        // labeled samples with owned Strings to avoid lifetime issues.
        let mut topic_offset_rows: Vec<(String, u64)> = Vec::new();
        let mut lag_rows: Vec<(String, String, u64)> = Vec::new();

        // Walk all subscriptions to build lag metrics.
        for subscription in journal.all_subscriptions() {
            let topic = &subscription.topic;
            let end_offset = journal.end_offset(topic);
            let escaped_topic = escape_label_value(topic);

            // tape_topic_latest_offset{topic} — collect once per topic,
            // dedup after loop.
            topic_offset_rows.push((escaped_topic.clone(), end_offset));

            // tape_subscription_lag{topic,subscription}
            let checkpoint = journal.checkpoint(topic, &subscription.name);
            let cursor_offset = checkpoint.map(|c| c.offset).unwrap_or(0);
            let lag = end_offset.saturating_sub(cursor_offset);

            let escaped_subscription = escape_label_value(&subscription.name);
            lag_rows.push((escaped_topic, escaped_subscription, lag));
        }

        // Deduplicate topic offsets (keep the first occurrence per topic).
        topic_offset_rows.sort_by(|a, b| a.0.cmp(&b.0));
        topic_offset_rows.dedup_by(|a, b| a.0 == b.0);

        // Render topic offsets if any subscriptions exist.
        if !topic_offset_rows.is_empty() {
            let topic_samples: Vec<LabeledSample> = topic_offset_rows
                .iter()
                .map(|(topic, offset)| {
                    LabeledSample::new(vec![Label::new("topic", topic)], *offset)
                })
                .collect();
            let topic_group = SampleGroup::new(
                "tape_topic_latest_offset",
                "gauge",
                "Latest offset for each topic (topic index + 1).",
                &topic_samples,
            );
            output.push_str(&metrics_prometheus::render_labeled(&[topic_group]));
        }

        // Render subscription lags if any subscriptions exist.
        if !lag_rows.is_empty() {
            let lag_samples: Vec<LabeledSample> = lag_rows
                .iter()
                .map(|(topic, subscription, lag)| {
                    LabeledSample::new(
                        vec![
                            Label::new("subscription", subscription),
                            Label::new("topic", topic),
                        ],
                        *lag,
                    )
                })
                .collect();
            let lag_group = SampleGroup::new(
                "tape_subscription_lag",
                "gauge",
                "Lag of each subscription's checkpoint behind the topic's latest offset.",
                &lag_samples,
            );
            output.push_str(&metrics_prometheus::render_labeled(&[lag_group]));
        }

        output
    }
}

// <HANDWRITE gap="missing-generator:public-peer-route-isolation" tracker="#1805" reason="public-peer-route-isolation section in server.rs is hand-written pending codegen support">
/// Build the HTTP router for the tape transport: the `/topics` data plane
/// merged onto the shared service shell's standard probe routes.
pub fn router(state: AppState) -> Router {
    router_with_admission(state, None)
}

/// Build the public router for a deployment whose Raft peer routes are owned
/// by the dedicated mTLS listener. The underlying application composition is
/// deliberately unchanged so data, probes, auth, admission, and metrics stay
/// identical; a first middleware rejects the two peer route families before
/// route dispatch can expose them on the public h2c listener.
pub fn router_without_raft_routes(state: AppState) -> Router {
    router_without_raft_routes_with_admission(state, None)
}

/// Build the secure-peer public router with optional shared request admission.
/// Peer route isolation stays outermost, so the public listener rejects Raft
/// routes before admission can account for them.
pub fn router_without_raft_routes_with_admission(
    state: AppState,
    admission: Option<service_http::AdmissionController>,
) -> Router {
    router_with_admission(state, admission).layer(from_fn(reject_public_raft_routes))
}

async fn reject_public_raft_routes(request: axum::extract::Request, next: Next) -> Response {
    let path = request.uri().path();
    if path == "/raftz" || path.starts_with("/raft/") {
        StatusCode::NOT_FOUND.into_response()
    } else {
        next.run(request).await
    }
}
// </HANDWRITE>

/// Build Tape with optional shared request admission. Tape owns the
/// read/write/admin route classes; `service-http` owns opaque-key retention,
/// token buckets, eviction, observability, and the 429 wire response.
pub fn router_with_admission(
    state: AppState,
    admission: Option<service_http::AdmissionController>,
) -> Router {
    let req_metrics = state.metrics();
    let verifier = state.verifier();
    let raft = state.raft();
    let body_limit = state.body_limit_bytes();
    let data_plane = Router::new()
        .route("/topics/{topic}/append", axum::routing::post(append))
        .route("/topics/{topic}/replay", get(replay))
        .route("/topics/{topic}/replay/stream", get(replay_stream))
        .route(
            "/topics/{topic}/consumers/{consumer}/checkpoint",
            get(checkpoint_get).put(checkpoint_put),
        )
        .route(
            "/topics/{topic}/subscriptions",
            get(subscription_list).post(subscription_create),
        )
        .route(
            "/topics/{topic}/subscriptions/{subscription}",
            get(subscription_get).delete(subscription_delete),
        )
        .route(
            "/topics/{topic}/subscriptions/{subscription}/pull",
            axum::routing::post(subscription_pull),
        )
        .route(
            "/topics/{topic}/subscriptions/{subscription}/ack",
            axum::routing::post(subscription_ack),
        )
        .route(
            "/topics/{topic}/retention",
            get(retention_get).put(retention_put),
        )
        // Cluster-wide admin op (#1329): a consistent snapshot of the journal
        // for backup runners. Inside the auth layer (unlike probes) — needs
        // `admin` on `*`.
        .route("/admin/backup", get(admin_backup))
        // Shared bearer auth (#1326) on the data plane ONLY — probes stay
        // tokenless. The blanket middleware authenticates (401 on a
        // missing/unknown token when required) and injects the
        // AuditedRoleMapPrincipal each handler authorizes on its {topic}.
        .route_layer(from_fn_with_state(
            verifier,
            service_auth::auth_middleware::<ReloadableRoleMapVerifier>,
        ))
        // Per-op request metrics (counts + latency). route_layer => only for
        // matched data-plane routes, and MatchedPath is populated. Added
        // after (= outside) the auth layer so rejected requests are still
        // counted.
        .route_layer(from_fn_with_state(req_metrics, crate::metrics::track))
        .with_state(state.clone())
        // Data-plane-only request body cap (#2484); probes below stay
        // unbounded, matching `service_http`'s documented probe behavior.
        // Enforces the configured body_limit_bytes with a structured 413 envelope.
        .layer(service_http::body_limit_layer(body_limit));
    let data_plane = match admission {
        Some(controller) => data_plane.route_layer(from_fn_with_state(
            service_http::AdmissionMiddleware::new(controller, |request| {
                let path = request.uri().path();
                let class = if path.starts_with("/admin/") {
                    "tape.admin"
                } else if *request.method() == Method::GET {
                    "tape.read"
                } else {
                    "tape.write"
                };
                let key = request
                    .headers()
                    .get(header::AUTHORIZATION)
                    .map(|value| value.as_bytes())
                    .unwrap_or(b"anonymous");
                Some(service_http::AdmissionInput::new(class, key))
            }),
            service_http::admission_middleware,
        )),
        None => data_plane,
    };

    // Standard probes (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
    // `/docs`) come from the shared service shell so the operational
    // surface matches every other service in the ecosystem. AppState
    // supplies readiness + Prometheus metrics; `/readyz` reports 503 while
    // draining.
    let probe_state = Arc::new(state);
    let metrics: Arc<dyn MetricsProvider> = probe_state.clone();
    let probes =
        service_http::standard_probe_routes(probe_state, Some(metrics), crate::openapi::openapi);

    let app = probes
        .merge(data_plane)
        // One INFO-level tracing span per request — spans probes + data plane.
        .layer(service_http::trace_layer())
        // Per-request Server-Timing response attribution, composed at the
        // same outermost position as trace_layer() above (#2490).
        .layer(from_fn(service_http::server_timing_middleware));

    // Peer raft RPCs + leader forward + `/raftz` (#1327) — merged OUTSIDE the
    // bearer-auth data plane, like the probes, since this is cluster traffic
    // between tape nodes rather than a client-facing route.
    match raft {
        Some(raft) => app.merge(raft.router()),
        None => app,
    }
}

/// Request body for `POST /topics/{topic}/append`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AppendRequest {
    /// Optional partitioning/idempotency key carried in the event envelope.
    #[serde(default)]
    pub key: Option<String>,
    /// Event payload.
    pub payload: serde_json::Value,
    /// Override event timestamp for deterministic tests/backfill.
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
}

/// Query params for `GET /topics/{topic}/replay`.
#[derive(Debug, Deserialize)]
pub struct ReplayQuery {
    #[serde(default)]
    pub from_offset: Option<u64>,
    #[serde(default)]
    pub from_timestamp_ms: Option<u64>,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Response body for `GET /topics/{topic}/replay`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReplayResponse {
    pub events: Vec<TapeEvent>,
}

/// Response body for `GET /topics/{topic}/consumers/{consumer}/checkpoint`.
#[derive(Debug, Serialize, ToSchema)]
pub struct CheckpointResponse {
    pub checkpoint: Option<ConsumerCheckpoint>,
}

/// Request body for `PUT /topics/{topic}/consumers/{consumer}/checkpoint`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckpointPutRequest {
    pub offset: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionCreateRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubscriptionPullRequest {
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubscriptionAckRequest {
    pub offset: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RetentionGetResponse {
    pub policy: Option<RetentionPolicy>,
}

/// `POST /topics/{topic}/append` — append one event envelope to the topic
/// journal.
#[utoipa::path(
    post,
    path = "/topics/{topic}/append",
    params(("topic" = String, Path, description = "Topic name")),
    request_body = AppendRequest,
    responses(
        (status = 200, description = "The appended event", body = TapeEvent),
        (status = 507, description = "Node is in ENOSPC degraded read-only mode (storage_full)")
    )
)]
pub async fn append(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Write) {
        return deny.into_response();
    }
    if let Err(full) = enforce_storage_writable(&st) {
        return full.into_response();
    }
    let req: AppendRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e.to_string())
                .into_response()
        }
    };
    // Resolve the timestamp BEFORE touching raft so every replica applies the
    // identical value (#1327) — same rule the direct-journal path already
    // follows via `TapeJournal::append`'s own `Option<u64>` -> `now_ms()`
    // fallback, just hoisted here so the proposed command carries it.
    let timestamp_ms = req.timestamp_ms.unwrap_or_else(crate::now_ms);
    // #3052 D3: stamp `applied_at_ms` here, in the handler, exactly like
    // `propose_append` does for the raft path -- passing `0` through would
    // make `apply_command`'s Append-branch fallback treat the CLIENT-
    // supplied `timestamp_ms` as the applied time, a silent behavior change.
    let applied_at_ms = crate::now_ms();

    if let Some(raft) = st.raft() {
        // append is NOT idempotent (unlike a message_id-keyed publish), so an
        // aged-out or failed outcome cannot be safely recomputed locally —
        // surface 503 rather than silently re-appending a possible duplicate.
        return match raft
            .propose_append(topic, req.key, req.payload, timestamp_ms)
            .await
        {
            Ok((_, Some(TapeOutcome::Appended(event)))) => {
                (StatusCode::OK, Json(event)).into_response()
            }
            Ok((_, Some(_))) => ApiErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "raft outcome kind mismatch for append",
            )
            .into_response(),
            Ok((_, None)) => ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "raft_unavailable",
                "append outcome aged out before this node could read it back",
            )
            .into_response(),
            Err(e) => ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "raft_unavailable",
                e.to_string(),
            )
            .into_response(),
        };
    }

    let command = TapeCommand::Append {
        topic,
        key: req.key,
        payload: req.payload,
        timestamp_ms,
        applied_at_ms,
    };
    match st.apply_mutation(command).await {
        Ok(TapeOutcome::Appended(event)) => (StatusCode::OK, Json(event)).into_response(),
        Ok(_) => outcome_mismatch("append"),
        Err(e) => persist_failure(e),
    }
}

/// `GET /topics/{topic}/replay` — replay topic history by offset or
/// timestamp.
#[utoipa::path(
    get,
    path = "/topics/{topic}/replay",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("from_offset" = Option<u64>, Query, description = "First offset to include"),
        ("from_timestamp_ms" = Option<u64>, Query, description = "First event timestamp to include"),
        ("limit" = Option<usize>, Query, description = "Maximum number of events to return"),
    ),
    responses((status = 200, description = "Matching events", body = ReplayResponse))
)]
pub async fn replay(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    Query(q): Query<ReplayQuery>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let journal = st.journal.lock().expect("journal mutex poisoned");
    let events = journal.replay(&topic, q.from_offset, q.from_timestamp_ms, q.limit);
    (StatusCode::OK, Json(ReplayResponse { events })).into_response()
}

/// `GET /topics/{topic}/replay/stream` — compact read-only h2c bulk replay.
/// The topic is carried by the path once; each frame retains offset, event
/// time, optional key, and opaque JSON payload bytes.
#[utoipa::path(
    get,
    path = "/topics/{topic}/replay/stream",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("from_offset" = Option<u64>, Query, description = "First offset to include"),
        ("from_timestamp_ms" = Option<u64>, Query, description = "First event timestamp to include"),
        ("limit" = Option<usize>, Query, description = "Maximum number of events to return"),
    ),
    responses((status = 200, description = "Length-framed Tape replay stream", content_type = "application/vnd.tape.replay.v1", body = Vec<u8>))
)]
pub async fn replay_stream(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    Query(q): Query<ReplayQuery>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let encoded = {
        let journal = st.journal.lock().expect("journal mutex poisoned");
        let events = journal.replay_refs(&topic, q.from_offset, q.from_timestamp_ms, q.limit);
        crate::replay_wire::encode(&events)
    };
    match encoded {
        Ok(body) => (
            [
                (header::CONTENT_TYPE, crate::replay_wire::CONTENT_TYPE),
                (header::CACHE_CONTROL, "no-store"),
            ],
            body,
        )
            .into_response(),
        Err(error) => ApiErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            error.to_string(),
        )
        .into_response(),
    }
}

/// `GET /topics/{topic}/consumers/{consumer}/checkpoint` — read a consumer
/// checkpoint.
#[utoipa::path(
    get,
    path = "/topics/{topic}/consumers/{consumer}/checkpoint",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("consumer" = String, Path, description = "Consumer name"),
    ),
    responses((status = 200, description = "The consumer's checkpoint, if any", body = CheckpointResponse))
)]
pub async fn checkpoint_get(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, consumer)): Path<(String, String)>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let journal = st.journal.lock().expect("journal mutex poisoned");
    let checkpoint = journal.checkpoint(&topic, &consumer).cloned();
    (StatusCode::OK, Json(CheckpointResponse { checkpoint })).into_response()
}

/// `PUT /topics/{topic}/consumers/{consumer}/checkpoint` — advance a
/// consumer checkpoint.
#[utoipa::path(
    put,
    path = "/topics/{topic}/consumers/{consumer}/checkpoint",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("consumer" = String, Path, description = "Consumer name"),
    ),
    request_body = CheckpointPutRequest,
    responses(
        (status = 200, description = "The advanced checkpoint", body = ConsumerCheckpoint),
        (status = 409, description = "Stale or beyond-end checkpoint offset"),
        (status = 507, description = "Node is in ENOSPC degraded read-only mode (storage_full)")
    )
)]
pub async fn checkpoint_put(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, consumer)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    if let Err(full) = enforce_storage_writable(&st) {
        return full.into_response();
    }
    let req: CheckpointPutRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e.to_string())
                .into_response()
        }
    };

    // Hoisted before both branches (raft AND the unified apply_mutation
    // path below) so both apply the identical `updated_at_ms` (#1327's rule,
    // now also required by #3052's shared `TapeCommand::CheckpointPut`).
    let updated_at_ms = crate::now_ms();
    if let Some(raft) = st.raft() {
        return match raft
            .propose_checkpoint(topic, consumer, req.offset, updated_at_ms)
            .await
        {
            Ok((_, Some(TapeOutcome::Checkpoint(Ok(checkpoint))))) => {
                (StatusCode::OK, Json(checkpoint)).into_response()
            }
            Ok((_, Some(TapeOutcome::Checkpoint(Err(e @ TapeError::StaleCheckpoint { .. }))))) => {
                ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response()
            }
            Ok((
                _,
                Some(TapeOutcome::Checkpoint(Err(e @ TapeError::CheckpointBeyondEnd { .. }))),
            )) => ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response(),
            Ok((_, Some(_))) => ApiErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "raft outcome kind mismatch for checkpoint_put",
            )
            .into_response(),
            Ok((_, None)) => ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "raft_unavailable",
                "checkpoint outcome aged out before this node could read it back",
            )
            .into_response(),
            Err(e) => ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "raft_unavailable",
                e.to_string(),
            )
            .into_response(),
        };
    }

    let command = TapeCommand::CheckpointPut {
        topic,
        consumer,
        offset: req.offset,
        updated_at_ms,
    };
    match st.apply_mutation(command).await {
        Ok(TapeOutcome::Checkpoint(Ok(checkpoint))) => {
            (StatusCode::OK, Json(checkpoint)).into_response()
        }
        Ok(TapeOutcome::Checkpoint(Err(e @ TapeError::StaleCheckpoint { .. }))) => {
            ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response()
        }
        Ok(TapeOutcome::Checkpoint(Err(e @ TapeError::CheckpointBeyondEnd { .. }))) => {
            ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response()
        }
        Ok(_) => outcome_mismatch("checkpoint_put"),
        Err(e) => persist_failure(e),
    }
}

#[utoipa::path(
    post,
    path = "/topics/{topic}/subscriptions",
    params(("topic" = String, Path, description = "Topic name")),
    request_body = SubscriptionCreateRequest,
    responses(
        (status = 201, description = "Created subscription", body = Subscription),
        (status = 507, description = "Node is in ENOSPC degraded read-only mode (storage_full)")
    )
)]
pub async fn subscription_create(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Write) {
        return deny.into_response();
    }
    if let Err(full) = enforce_storage_writable(&st) {
        return full.into_response();
    }
    let req: SubscriptionCreateRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", error.to_string())
                .into_response()
        }
    };
    if let Some(raft) = st.raft() {
        return match raft.propose_subscription_create(topic, req.name).await {
            Ok((_, Some(TapeOutcome::SubscriptionCreated(Ok(subscription))))) => {
                (StatusCode::CREATED, Json(subscription)).into_response()
            }
            Ok((_, Some(TapeOutcome::SubscriptionCreated(Err(error))))) => {
                subscription_error(error)
            }
            Ok((_, Some(_))) => outcome_mismatch("subscription_create"),
            Ok((_, None)) => missing_outcome("subscription_create"),
            Err(error) => raft_unavailable(error),
        };
    }
    let command = TapeCommand::SubscriptionCreate {
        topic,
        name: req.name,
    };
    match st.apply_mutation(command).await {
        Ok(TapeOutcome::SubscriptionCreated(Ok(subscription))) => {
            (StatusCode::CREATED, Json(subscription)).into_response()
        }
        Ok(TapeOutcome::SubscriptionCreated(Err(error))) => subscription_error(error),
        Ok(_) => outcome_mismatch("subscription_create"),
        Err(error) => persist_failure(error),
    }
}

#[utoipa::path(
    get,
    path = "/topics/{topic}/subscriptions",
    params(("topic" = String, Path, description = "Topic name")),
    responses((status = 200, description = "Topic subscriptions", body = SubscriptionListResponse))
)]
pub async fn subscription_list(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let subscriptions = st
        .journal
        .lock()
        .expect("journal mutex poisoned")
        .subscriptions(&topic);
    (
        StatusCode::OK,
        Json(SubscriptionListResponse { subscriptions }),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/topics/{topic}/subscriptions/{subscription}",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("subscription" = String, Path, description = "Subscription name")
    ),
    responses((status = 200, description = "Subscription", body = Subscription))
)]
pub async fn subscription_get(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, name)): Path<(String, String)>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    match st
        .journal
        .lock()
        .expect("journal mutex poisoned")
        .subscription(&topic, &name)
        .cloned()
    {
        Some(subscription) => (StatusCode::OK, Json(subscription)).into_response(),
        None => subscription_error(SubscriptionError::NotFound { topic, name }),
    }
}

#[utoipa::path(
    delete,
    path = "/topics/{topic}/subscriptions/{subscription}",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("subscription" = String, Path, description = "Subscription name")
    ),
    responses(
        (status = 200, description = "Deleted subscription", body = Subscription),
        (status = 507, description = "Node is in ENOSPC degraded read-only mode (storage_full)")
    )
)]
pub async fn subscription_delete(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, name)): Path<(String, String)>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Write) {
        return deny.into_response();
    }
    // A delete shrinks the journal, so it is tempting to exempt it — but the
    // persist that follows still rewrites the WHOLE journal through a temp
    // file, which needs room for a second copy before the old one is unlinked.
    // On a full disk a delete fails exactly like an append.
    if let Err(full) = enforce_storage_writable(&st) {
        return full.into_response();
    }
    if let Some(raft) = st.raft() {
        return match raft.propose_subscription_delete(topic, name).await {
            Ok((_, Some(TapeOutcome::SubscriptionDeleted(Ok(subscription))))) => {
                (StatusCode::OK, Json(subscription)).into_response()
            }
            Ok((_, Some(TapeOutcome::SubscriptionDeleted(Err(error))))) => {
                subscription_error(error)
            }
            Ok((_, Some(_))) => outcome_mismatch("subscription_delete"),
            Ok((_, None)) => missing_outcome("subscription_delete"),
            Err(error) => raft_unavailable(error),
        };
    }
    let command = TapeCommand::SubscriptionDelete { topic, name };
    match st.apply_mutation(command).await {
        Ok(TapeOutcome::SubscriptionDeleted(Ok(subscription))) => {
            (StatusCode::OK, Json(subscription)).into_response()
        }
        Ok(TapeOutcome::SubscriptionDeleted(Err(error))) => subscription_error(error),
        Ok(_) => outcome_mismatch("subscription_delete"),
        Err(error) => persist_failure(error),
    }
}

#[utoipa::path(
    post,
    path = "/topics/{topic}/subscriptions/{subscription}/pull",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("subscription" = String, Path, description = "Subscription name")
    ),
    request_body = Option<SubscriptionPullRequest>,
    responses((status = 200, description = "Side-effect-free bounded replay window", body = PullSubscriptionBatch))
)]
pub async fn subscription_pull(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, name)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let req: SubscriptionPullRequest = if body.is_empty() {
        SubscriptionPullRequest { limit: None }
    } else {
        match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", error.to_string())
                    .into_response()
            }
        }
    };
    match st
        .journal
        .lock()
        .expect("journal mutex poisoned")
        .pull_subscription(&topic, &name, req.limit)
    {
        Ok(batch) => (StatusCode::OK, Json::<PullSubscriptionBatch>(batch)).into_response(),
        Err(error) => subscription_error(error),
    }
}

#[utoipa::path(
    post,
    path = "/topics/{topic}/subscriptions/{subscription}/ack",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("subscription" = String, Path, description = "Subscription name")
    ),
    request_body = SubscriptionAckRequest,
    responses(
        (status = 200, description = "Explicitly advanced checkpoint", body = ConsumerCheckpoint),
        (status = 507, description = "Node is in ENOSPC degraded read-only mode (storage_full)")
    )
)]
pub async fn subscription_ack(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, name)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    if let Err(full) = enforce_storage_writable(&st) {
        return full.into_response();
    }
    let req: SubscriptionAckRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", error.to_string())
                .into_response()
        }
    };
    // Hoisted before both branches, same rule as checkpoint_put above:
    // raft and the unified apply_mutation path must apply the identical
    // `updated_at_ms`.
    let updated_at_ms = crate::now_ms();
    if let Some(raft) = st.raft() {
        return match raft
            .propose_subscription_ack(topic, name, req.offset, updated_at_ms)
            .await
        {
            Ok((_, Some(TapeOutcome::SubscriptionAcked(Ok(checkpoint))))) => {
                (StatusCode::OK, Json(checkpoint)).into_response()
            }
            Ok((_, Some(TapeOutcome::SubscriptionAcked(Err(error))))) => {
                subscription_ack_error(error)
            }
            Ok((_, Some(_))) => outcome_mismatch("subscription_ack"),
            Ok((_, None)) => missing_outcome("subscription_ack"),
            Err(error) => raft_unavailable(error),
        };
    }
    let command = TapeCommand::SubscriptionAck {
        topic,
        name,
        offset: req.offset,
        updated_at_ms,
    };
    match st.apply_mutation(command).await {
        Ok(TapeOutcome::SubscriptionAcked(Ok(checkpoint))) => {
            (StatusCode::OK, Json(checkpoint)).into_response()
        }
        Ok(TapeOutcome::SubscriptionAcked(Err(error))) => subscription_ack_error(error),
        Ok(_) => outcome_mismatch("subscription_ack"),
        Err(error) => persist_failure(error),
    }
}

fn subscription_error(error: SubscriptionError) -> Response {
    let status = match error {
        SubscriptionError::NotFound { .. } => StatusCode::NOT_FOUND,
        SubscriptionError::AlreadyExists { .. } => StatusCode::CONFLICT,
        SubscriptionError::PullBatchTooLarge { .. } => StatusCode::BAD_REQUEST,
    };
    ApiErr::new(status, "subscription_error", error.to_string()).into_response()
}

fn subscription_ack_error(error: SubscriptionAckError) -> Response {
    match error {
        SubscriptionAckError::Subscription(error) => subscription_error(error),
        SubscriptionAckError::Checkpoint(error) => {
            ApiErr::new(StatusCode::CONFLICT, "conflict", error.to_string()).into_response()
        }
    }
}

fn raft_unavailable(error: anyhow::Error) -> Response {
    ApiErr::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "raft_unavailable",
        error.to_string(),
    )
    .into_response()
}

fn missing_outcome(operation: &str) -> Response {
    ApiErr::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "raft_unavailable",
        format!("{operation} outcome unavailable after commit"),
    )
    .into_response()
}

fn outcome_mismatch(operation: &str) -> Response {
    ApiErr::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        format!("raft outcome kind mismatch for {operation}"),
    )
    .into_response()
}

#[utoipa::path(
    get,
    path = "/topics/{topic}/retention",
    params(("topic" = String, Path, description = "Topic name")),
    responses((status = 200, description = "Topic retention policy", body = RetentionGetResponse))
)]
pub async fn retention_get(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let policy = st
        .journal
        .lock()
        .expect("journal mutex poisoned")
        .retention(&topic)
        .cloned();
    (StatusCode::OK, Json(RetentionGetResponse { policy })).into_response()
}

#[utoipa::path(
    put,
    path = "/topics/{topic}/retention",
    params(("topic" = String, Path, description = "Topic name")),
    request_body = RetentionPolicy,
    responses(
        (status = 200, description = "Applied policy and compaction result", body = RetentionOutcome),
        (status = 507, description = "Node is in ENOSPC degraded read-only mode (storage_full)")
    )
)]
pub async fn retention_put(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Write) {
        return deny.into_response();
    }
    // Same reasoning as `subscription_delete`: applying retention compacts the
    // journal, but the persist that records it still stages a full second copy
    // first. Freeing space is not a way around a disk that is already full.
    if let Err(full) = enforce_storage_writable(&st) {
        return full.into_response();
    }
    let policy: RetentionPolicy = match serde_json::from_slice(&body) {
        Ok(policy) => policy,
        Err(error) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", error.to_string())
                .into_response()
        }
    };
    let now_ms = crate::now_ms();
    if let Some(raft) = st.raft() {
        return match raft.propose_retention(topic, policy, now_ms).await {
            Ok((_, Some(TapeOutcome::RetentionUpdated(outcome)))) => {
                (StatusCode::OK, Json(outcome)).into_response()
            }
            Ok((_, Some(_))) => outcome_mismatch("retention_put"),
            Ok((_, None)) => missing_outcome("retention_put"),
            Err(error) => raft_unavailable(error),
        };
    }
    let command = TapeCommand::RetentionPut {
        topic,
        policy,
        now_ms,
    };
    match st.apply_mutation(command).await {
        Ok(TapeOutcome::RetentionUpdated(outcome)) => {
            (StatusCode::OK, Json(outcome)).into_response()
        }
        Ok(_) => outcome_mismatch("retention_put"),
        Err(error) => persist_failure(error),
    }
}

/// `GET /admin/backup` — a consistent snapshot of the whole journal for
/// backup runners (#1329): the EXACT bytes [`crate::raft::TapeStateMachine`]'s
/// raft snapshot produces ([`crate::raft::snapshot_bytes`] — the whole
/// journal + the applied index; 0 on a raft-less single node). A
/// cluster-wide admin op: requires `admin` on `*` when auth is required.
/// Restore = feed the bytes to `TapeStateMachine::restore` on a fresh node
/// (the existing raft-side merge path); no restore CLI verb is added here.
#[utoipa::path(
    get,
    path = "/admin/backup",
    responses((status = 200, description = "JournalSnapshot JSON { up_to, journal } — the whole journal at the applied raft index"))
)]
pub async fn admin_backup(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, "*", Role::Admin) {
        return deny.into_response();
    }
    let raft = st.raft();
    let applied = raft.as_ref().map(|raft| raft.applied_index()).unwrap_or(0);
    let snapshot = match raft {
        Some(raft) => raft.snapshot_bytes(),
        None => crate::raft::snapshot_bytes(&st.journal, applied),
    };
    match snapshot {
        Ok(bytes) => {
            // Audit only the low-frequency management operation. Append and
            // consumer checkpoint traffic is deliberately not duplicated into
            // logs: its durable, payload-free audit trail is the Tape journal
            // itself, while credentials/denials are already emitted through
            // the shared service-auth redacted audit sink.
            tracing::info!(
                target: "tape.audit",
                event = "backup_snapshot_served",
                subject = principal.subject().unwrap_or("anonymous"),
                applied_index = applied,
                bytes = bytes.len(),
            );
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                bytes,
            )
                .into_response()
        }
        Err(e) => ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn append_replay_and_checkpoint_round_trip() {
        let state = AppState::new(TapeJournal::default(), None, 8 * 1024 * 1024);
        let app = router(state);

        let resp = crate::server::tests::post_json(
            app.clone(),
            "/topics/orders/append",
            &serde_json::json!({ "payload": { "n": 1 } }),
        )
        .await;
        assert_eq!(resp.0, StatusCode::OK);

        let resp = crate::server::tests::get(app.clone(), "/topics/orders/replay").await;
        assert_eq!(resp.0, StatusCode::OK);
        let body: serde_json::Value = serde_json::from_str(&resp.1).unwrap();
        assert_eq!(body["events"].as_array().unwrap().len(), 1);

        let resp = crate::server::tests::put_json(
            app.clone(),
            "/topics/orders/consumers/c1/checkpoint",
            &serde_json::json!({ "offset": 1 }),
        )
        .await;
        assert_eq!(resp.0, StatusCode::OK);

        let resp =
            crate::server::tests::get(app.clone(), "/topics/orders/consumers/c1/checkpoint").await;
        assert_eq!(resp.0, StatusCode::OK);
        let body: serde_json::Value = serde_json::from_str(&resp.1).unwrap();
        assert_eq!(body["checkpoint"]["offset"], 1);
    }

    #[tokio::test]
    async fn pull_subscription_is_bounded_side_effect_free_and_explicitly_acked() {
        let app = router(AppState::new(TapeJournal::default(), None, 8 * 1024 * 1024));
        for n in 0..2 {
            let response = post_json(
                app.clone(),
                "/topics/orders/append",
                &serde_json::json!({ "payload": { "n": n } }),
            )
            .await;
            assert_eq!(response.0, StatusCode::OK);
        }
        let created = post_json(
            app.clone(),
            "/topics/orders/subscriptions",
            &serde_json::json!({ "name": "audit" }),
        )
        .await;
        assert_eq!(created.0, StatusCode::CREATED);

        let push = post_json(
            app.clone(),
            "/topics/orders/subscriptions",
            &serde_json::json!({
                "name": "webhook",
                "delivery": { "mode": "push", "endpoint": "https://example.invalid" }
            }),
        )
        .await;
        assert_eq!(push.0, StatusCode::BAD_REQUEST, "push is not a Tape mode");

        let first = post_json(
            app.clone(),
            "/topics/orders/subscriptions/audit/pull",
            &serde_json::json!({ "limit": 2 }),
        )
        .await;
        assert_eq!(first.0, StatusCode::OK);
        let first_body: serde_json::Value = serde_json::from_str(&first.1).unwrap();
        assert_eq!(first_body["cursor"], 0);
        assert_eq!(first_body["next_offset"], 2);
        assert_eq!(first_body["events"].as_array().unwrap().len(), 2);

        let repeated = post_json(
            app.clone(),
            "/topics/orders/subscriptions/audit/pull",
            &serde_json::json!({ "limit": 2 }),
        )
        .await;
        let repeated_body: serde_json::Value = serde_json::from_str(&repeated.1).unwrap();
        assert_eq!(repeated_body["cursor"], 0, "pull must not implicitly ack");

        let acked = post_json(
            app.clone(),
            "/topics/orders/subscriptions/audit/ack",
            &serde_json::json!({ "offset": 2 }),
        )
        .await;
        assert_eq!(acked.0, StatusCode::OK);

        let drained = post_json(
            app.clone(),
            "/topics/orders/subscriptions/audit/pull",
            &serde_json::json!({ "limit": 2 }),
        )
        .await;
        let drained_body: serde_json::Value = serde_json::from_str(&drained.1).unwrap();
        assert_eq!(drained_body["cursor"], 2);
        assert!(drained_body["events"].as_array().unwrap().is_empty());

        let stale = post_json(
            app,
            "/topics/orders/subscriptions/audit/ack",
            &serde_json::json!({ "offset": 1 }),
        )
        .await;
        assert_eq!(stale.0, StatusCode::CONFLICT);
    }

    /// R1: `GET /admin/backup` denies a non-admin principal (403) and
    /// streams exactly the `raft::snapshot_bytes` bytes to an admin-on-`*`
    /// principal (200), over an in-process `oneshot` request (no real
    /// socket — `e2e/backup.rs` covers the live-HTTP + 401 case).
    #[tokio::test]
    async fn admin_backup_requires_admin_and_streams_snapshot() {
        use tower::ServiceExt;
        let tokens = serde_json::json!({
            "admin-token": { "subject": "ops", "roles": { "*": "admin" } },
            "reader-token": { "subject": "worker", "roles": { "*": "read" } },
        })
        .to_string();
        let auth = crate::auth::AuthConfig::resolve("required", None, Some(&tokens)).unwrap();
        let mut journal = TapeJournal::default();
        journal.append("orders", None, serde_json::json!({"n": 1}), Some(100));
        let state = AppState::with_auth(journal, None, auth, 8 * 1024 * 1024);
        let handle = state.journal_handle();
        let app = router(state);

        let deny = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/admin/backup")
                    .header("authorization", "Bearer reader-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deny.status(), StatusCode::FORBIDDEN);

        let ok = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/admin/backup")
                    .header("authorization", "Bearer admin-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        let bytes = http_body_util::BodyExt::collect(ok.into_body())
            .await
            .unwrap()
            .to_bytes();
        let expected = crate::raft::snapshot_bytes(&handle, 0).unwrap();
        assert_eq!(&bytes[..], &expected[..]);
    }

    #[tokio::test]
    async fn secure_peer_mode_does_not_expose_raft_routes_on_public_router() {
        use tower::ServiceExt;

        let journal = Arc::new(Mutex::new(TapeJournal::default()));
        let dir = tempfile::tempdir().unwrap();
        let raft = Arc::new(
            TapeRaft::spawn(
                Arc::clone(&journal),
                dir.path(),
                0,
                raft_runtime::Membership {
                    voters: vec![0],
                    learners: vec![],
                },
                std::collections::HashMap::new(),
                TapeRaft::host_config(1024),
            )
            .unwrap(),
        );
        let mut state = AppState::new(TapeJournal::default(), None, 8 * 1024 * 1024);
        state.set_raft(raft);

        let response = router_without_raft_routes(state)
            .oneshot(
                axum::http::Request::builder()
                    .uri("/raftz")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Small oneshot helpers so both this module's tests and
    // `e2e/http_transport.rs` share one shape (the integration test drives
    // the router over real HTTP instead — these stay unit-level).
    pub(crate) async fn get(app: Router, path: &str) -> (StatusCode, String) {
        use http_body_util::BodyExt;
        use tower::ServiceExt;
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .uri(path)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    pub(crate) async fn post_json(
        app: Router,
        path: &str,
        body: &serde_json::Value,
    ) -> (StatusCode, String) {
        use http_body_util::BodyExt;
        use tower::ServiceExt;
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    pub(crate) async fn put_json(
        app: Router,
        path: &str,
        body: &serde_json::Value,
    ) -> (StatusCode, String) {
        use http_body_util::BodyExt;
        use tower::ServiceExt;
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("PUT")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    /// Build a state whose `--store` points at `path`, carrying one event on
    /// `topic` so the persisted journal has content worth losing.
    fn state_with_store(path: &std::path::Path, topic: &str) -> AppState {
        let mut journal = TapeJournal::default();
        journal.append(topic, None, serde_json::json!({ "n": 1 }), None);
        AppState::new(journal, Some(path.to_path_buf()), 8 * 1024 * 1024)
    }

    /// The staging path `storage_durable::atomic_write` uses for `path` —
    /// the whole path with `.tmp` appended, not an extension swap.
    fn staging_path(path: &std::path::Path) -> std::path::PathBuf {
        let mut tmp = path.as_os_str().to_os_string();
        tmp.push(".tmp");
        tmp.into()
    }

    /// #2572 — a failed write must leave the PREVIOUS journal intact.
    ///
    /// The failure is induced by occupying the temp path with a directory, so
    /// `atomic_write` cannot create its temp file. That is a stand-in for the
    /// real motivating cases (crash mid-write, pod eviction, ENOSPC) which are
    /// not deterministically reproducible in a unit test — what it reproduces
    /// faithfully is the property under test: the write fails *before* the
    /// live file is touched.
    ///
    /// Verified to fail against the pre-#2572 implementation, at the
    /// `expect_err`: `fs::write` ignores the staging path entirely, so it
    /// reported success and replaced the journal it could not safely write.
    #[test]
    fn persist_failure_leaves_the_previous_journal_intact() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("journal.json");

        // Establish a good journal on disk.
        let first = state_with_store(&path, "orders");
        first
            .persist(&first.journal.lock().unwrap())
            .expect("first persist writes the journal");
        let good = std::fs::read(&path).expect("journal is on disk");
        assert!(!good.is_empty());

        // Block the temp path, then try to persist different content.
        std::fs::create_dir(staging_path(&path)).unwrap();
        let second = state_with_store(&path, "shipments");
        let error = second
            .persist(&second.journal.lock().unwrap())
            .expect_err("persist must fail when it cannot stage the write");

        // The live journal is byte-identical and still parses.
        assert_eq!(
            std::fs::read(&path).unwrap(),
            good,
            "a failed persist must not modify the live journal; error was: {error}"
        );
        let reloaded: TapeJournal = serde_json::from_slice(&good).expect("journal still parses");
        assert_eq!(
            reloaded.replay("orders", None, None, None).len(),
            1,
            "the surviving journal is the original one, not the failed write"
        );
        assert!(
            reloaded.replay("shipments", None, None, None).is_empty(),
            "the failed write left no trace in the live journal"
        );
    }

    /// #2572 — a successful persist commits by rename and leaves no residue.
    /// A leftover `.tmp` would mean the rename did not happen and the next
    /// boot could find two candidate files.
    ///
    /// Unlike the test above this one also passed pre-#2572 (`fs::write`
    /// never creates a staging file to leave behind). It is not a regression
    /// guard for the old bug; it guards the new mechanism — that the commit
    /// path stays a rename, and that parent-directory creation survived the
    /// move into `atomic_write`.
    #[test]
    fn persist_commits_by_rename_without_temp_residue() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("journal.json");

        let state = state_with_store(&path, "orders");
        state
            .persist(&state.journal.lock().unwrap())
            .expect("persist creates parent directories and writes");

        assert!(path.exists(), "the journal is at its final path");
        assert!(
            !staging_path(&path).exists(),
            "the temp file was renamed into place, not left behind"
        );
        let reloaded: TapeJournal = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(reloaded.replay("orders", None, None, None).len(), 1);
    }

    /// #2573 AC1-AC3 — the whole degraded-mode contract in one journey: the
    /// first ENOSPC answers a typed 507 and latches degraded mode, every
    /// later mutation is fast-failed *before* the journal is touched, and
    /// reads keep serving throughout.
    ///
    /// The ENOSPC itself comes from the `#[cfg(test)]` injection seam rather
    /// than a real full disk: filling a tmpfs from a unit test is neither
    /// hermetic nor portable, and what is under test is the reaction to
    /// `io::ErrorKind::StorageFull`, not the kernel's ability to produce it.
    /// #2572 is what makes the seam faithful — it preserves the ErrorKind
    /// through `atomic_write`'s context chain, so the real path reaches this
    /// same branch with the same kind.
    /// `apply_mutation` latches degraded mode on ENOSPC *or* EIO (WI #3052
    /// R6). ENOSPC is covered end to end by the injection-seam test below;
    /// this pins the EIO half of the predicate, which is the half with no
    /// stable `ErrorKind` to ride on and which was silently unreachable
    /// before `wal::DurabilityFailure` carried the errno explicitly.
    #[test]
    fn eio_is_recognized_through_the_durability_error_the_wal_path_actually_produces() {
        const EIO: i32 = 5;
        let from_wal_path =
            crate::wal::flatten_io_error_for_test(std::io::Error::from_raw_os_error(EIO));
        assert!(is_eio(&from_wal_path));

        // A plain failure must NOT latch degraded mode: it fails one request
        // closed and leaves the node writable.
        assert!(!is_eio(&std::io::Error::other("transient")));
        assert!(!is_eio(&std::io::Error::from(
            std::io::ErrorKind::PermissionDenied
        )));
    }

    #[tokio::test]
    async fn enospc_latches_degraded_mode_fast_fails_mutations_and_keeps_reads_serving() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("journal.json");
        let state = state_with_store(&path, "orders");
        let app = router(state.clone());

        // Baseline: a healthy node appends and is not degraded.
        let healthy = post_json(
            app.clone(),
            "/topics/orders/append",
            &serde_json::json!({ "payload": { "n": 2 } }),
        )
        .await;
        assert_eq!(healthy.0, StatusCode::OK);
        assert!(!state.metrics.is_storage_degraded());

        // AC1: the persist that hits ENOSPC answers 507 with the typed
        // `storage_full` kind — not a generic 500 a client would retry into.
        state.set_inject_storage_full(true);
        let first = post_json(
            app.clone(),
            "/topics/orders/append",
            &serde_json::json!({ "payload": { "n": 3 } }),
        )
        .await;
        assert_eq!(first.0, StatusCode::INSUFFICIENT_STORAGE);
        assert!(
            first.1.contains("storage_full"),
            "the error kind must be typed, got: {}",
            first.1
        );
        assert!(state.metrics.is_storage_degraded(), "the flag is sticky");
        assert_eq!(state.metrics.storage_full_errors_total.get(), 1);

        // AC2: further mutations short-circuit at the gate. The event count
        // staying put is the proof they never reached the journal — a gate
        // that ran after the mutation would leave the in-memory journal
        // drifting ahead of the durable one on every rejected request.
        let events_after_first_failure = state
            .journal
            .lock()
            .unwrap()
            .replay("orders", None, None, None)
            .len();
        for (label, response) in [
            (
                "append",
                post_json(
                    app.clone(),
                    "/topics/orders/append",
                    &serde_json::json!({ "payload": { "n": 4 } }),
                )
                .await,
            ),
            (
                "checkpoint advance",
                put_json(
                    app.clone(),
                    "/topics/orders/consumers/c1/checkpoint",
                    &serde_json::json!({ "offset": 1 }),
                )
                .await,
            ),
            (
                "subscription create",
                post_json(
                    app.clone(),
                    "/topics/orders/subscriptions",
                    &serde_json::json!({ "name": "audit" }),
                )
                .await,
            ),
            (
                "retention set",
                put_json(
                    app.clone(),
                    "/topics/orders/retention",
                    &serde_json::json!({ "max_events": 10 }),
                )
                .await,
            ),
        ] {
            assert_eq!(
                response.0,
                StatusCode::INSUFFICIENT_STORAGE,
                "{label} must be fast-failed while degraded, got: {}",
                response.1
            );
        }
        assert_eq!(
            state
                .journal
                .lock()
                .unwrap()
                .replay("orders", None, None, None)
                .len(),
            events_after_first_failure,
            "fast-failed mutations must not touch the journal"
        );
        assert_eq!(
            state.metrics.storage_full_errors_total.get(),
            1,
            "the gate never reaches the durable path, so it counts no new ENOSPC hits"
        );

        // AC3: degraded is read-ONLY, not down. Replay keeps answering — a
        // full disk is precisely when an operator most needs to read what is
        // already journalled.
        let replay = get(app, "/topics/orders/replay").await;
        assert_eq!(replay.0, StatusCode::OK);
    }

    /// #2573 AC4 (unit half) — clearing degraded mode returns the node to
    /// normal service with no restart. This drives `clear_storage_degraded`
    /// directly, which is exactly what the periodic re-probe task in
    /// `src/bin/tape.rs` (`spawn_storage_full_reprobe`) calls once a probe
    /// write into the store directory succeeds; the timer itself is not worth
    /// a 30s unit test.
    #[tokio::test]
    async fn leaving_degraded_mode_restores_mutations_without_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("journal.json");
        let state = state_with_store(&path, "orders");
        let app = router(state.clone());

        state.set_inject_storage_full(true);
        let blocked = post_json(
            app.clone(),
            "/topics/orders/append",
            &serde_json::json!({ "payload": { "n": 2 } }),
        )
        .await;
        assert_eq!(blocked.0, StatusCode::INSUFFICIENT_STORAGE);

        // The disk got bigger / something got freed; the re-probe succeeds.
        state.set_inject_storage_full(false);
        state.metrics.clear_storage_degraded();

        let recovered = post_json(
            app,
            "/topics/orders/append",
            &serde_json::json!({ "payload": { "n": 3 } }),
        )
        .await;
        assert_eq!(recovered.0, StatusCode::OK, "same process, no restart");
        assert!(path.exists(), "and the journal is durable again");
        assert_eq!(
            state.metrics.storage_full_errors_total.get(),
            1,
            "recovery does not erase the record that this node was once full"
        );
    }
}
