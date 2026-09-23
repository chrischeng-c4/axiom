// CODEGEN-BEGIN
//! Lightweight in-process Prometheus exposition.
//!
//! v1 keeps the metric surface narrow and dep-free: a handful of
//! `AtomicU64` counters/gauges + a single `render` that emits the
//! Prometheus text-format. When request volume grows past what
//! lock-free counters can serve, swap in `prometheus`/`metrics` crates
//! without changing the wire format the scraper sees.
//!
//! The counter/gauge primitives and the Prometheus text-format encoder
//! are generic across every service with a `/metrics` scrape endpoint
//! and live in `libs/metrics-prometheus` (#974); this module wires lumen's
//! metric names, HELP text, and `# TYPE` kinds onto them. `Counter`/
//! `Gauge` deref to the underlying `AtomicU64`, so this module's pub API
//! — field names, method names, and `render()`'s byte output — is
//! unchanged for callers, including the `otel` feature's direct
//! `field.load(Ordering::Relaxed)` reads in `src/bin/lumen.rs`.

use crate::change_budget::ChangeBudget;
use metrics_prometheus::{Counter, Gauge, Label, LabeledSample, Sample, SampleGroup};
use std::fmt::Write as _;
use std::time::Duration;

/// #2475: sentinel `raft_shard` value meaning "never touched by the raft
/// election-state poller" — see [`Metrics::raft_shard`]. Out of range for
/// any real shard index, so it stays distinguishable from every real shard.
const NOT_RAFT: u64 = u64::MAX;

/// #2519: number of finite `lumen_search_latency_seconds_bucket{le=...}`
/// rows — see [`SEARCH_LATENCY_BUCKETS_US`].
const SEARCH_LATENCY_BUCKET_COUNT: usize = 12;

/// #2519: search-latency histogram bucket upper bounds, sized for search
/// SLOs from a sub-millisecond fast path up to a 5s outlier tail. Each pair
/// is `(the Prometheus "le" label, the same bound in whole microseconds)` —
/// microseconds because every bound here is an exact conversion of a "nice"
/// decimal-seconds SLO value (e.g. `0.0025s == 2_500us`), so bucket
/// assignment in [`Metrics::observe_search`] is exact integer comparison,
/// never float rounding. Bucket `i` (see `search_latency_buckets`) counts
/// observations in `(bound[i-1], bound[i]]`, with `bound[-1] == 0`.
/// Observations past the last bound aren't stored in a 13th counter — the
/// Prometheus `+Inf` bucket renders as the plain total observation count
/// instead (see `render_search_latency_histogram`).
const SEARCH_LATENCY_BUCKETS_US: [(&str, u64); SEARCH_LATENCY_BUCKET_COUNT] = [
    ("0.001", 1_000),
    ("0.0025", 2_500),
    ("0.005", 5_000),
    ("0.01", 10_000),
    ("0.025", 25_000),
    ("0.05", 50_000),
    ("0.1", 100_000),
    ("0.25", 250_000),
    ("0.5", 500_000),
    ("1", 1_000_000),
    ("2.5", 2_500_000),
    ("5", 5_000_000),
];

/// #2519: `lumen_slow_queries_total`'s threshold (milliseconds) when
/// `LUMEN_SLOW_QUERY_MS` is unset or unparseable — see
/// `slow_query_threshold_ms_from_env`.
const DEFAULT_SLOW_QUERY_THRESHOLD_MS: u64 = 500;

/// Number of [`MergeStep`] variants — the row count of the
/// `lumen_segment_merge_phase_seconds{phase=...}` family.
pub const MERGE_STEP_COUNT: usize = 10;

/// One timed step of a single background segment merge job.
///
/// A merge job's cost is not one number: the payload work
/// (`Compact`) is proportional to the fields it drains, while the
/// generation-wide hard-link, layout-validation, inheritance, and capture
/// steps are proportional to the *whole* root — every file of every
/// collection, drained or idle. Splitting the job into named steps is what
/// lets a production `/metrics` scrape say which of the two dominates,
/// instead of leaving that to a guess about a job that took seconds.
///
/// Ordering is the order the steps run inside
/// `crate::segment_background_merge::SegmentRdbStore::merge_one`, and
/// [`MergeStep::Total`] spans the whole job so
/// `Total - sum(others)` is the unattributed remainder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MergeStep {
    /// `engine.capture_background_merge(identities(&prior))` taken under
    /// the first `save_gate` hold, before any file work.
    CaptureBefore,
    /// `link_collection(source -> scratch)`: hard-links the files of the one
    /// collection this job compacts into its job-private scratch stage. The
    /// count is proportional to the job's payload, not to the root.
    LinkScratch,
    /// `compact_staged_delta_windows`: `compact_one_staged_field` for the one
    /// field this job compacts. The only payload-proportional step.
    Compact,
    /// `link_collections_with_paths(latest -> new generation)`: the second
    /// whole-root hard-link pass, taken while `save_gate` is held.
    LinkGeneration,
    /// `validate_generation_layout_with_prior`.
    ValidateLayout,
    /// The `inherit_current_file` loop over every inherited file.
    InheritFiles,
    /// `telemetry::pending_deltas` over the published catalog.
    PendingDeltas,
    /// The publication-guard `capture_background_merge` re-capture.
    CapturePublish,
    /// `write_generation_manifest` for the rebased catalog.
    ManifestWrite,
    /// The whole job, from the first `save_gate` acquisition to the last
    /// `drop(guard)`.
    Total,
}

impl MergeStep {
    /// Every step, in run order. Rendering iterates this, so the published
    /// row set is fixed and a scrape never silently loses a phase.
    pub const ALL: [MergeStep; MERGE_STEP_COUNT] = [
        MergeStep::CaptureBefore,
        MergeStep::LinkScratch,
        MergeStep::Compact,
        MergeStep::LinkGeneration,
        MergeStep::ValidateLayout,
        MergeStep::InheritFiles,
        MergeStep::PendingDeltas,
        MergeStep::CapturePublish,
        MergeStep::ManifestWrite,
        MergeStep::Total,
    ];

    /// The `phase` label value published for this step.
    pub const fn name(self) -> &'static str {
        match self {
            MergeStep::CaptureBefore => "capture_before",
            MergeStep::LinkScratch => "link_scratch",
            MergeStep::Compact => "compact",
            MergeStep::LinkGeneration => "link_generation",
            MergeStep::ValidateLayout => "validate_layout",
            MergeStep::InheritFiles => "inherit_files",
            MergeStep::PendingDeltas => "pending_deltas",
            MergeStep::CapturePublish => "capture_publish",
            MergeStep::ManifestWrite => "manifest_write",
            MergeStep::Total => "total",
        }
    }

    /// This step's slot in the per-step atomic arrays.
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// #4326: number of finite `lumen_coordinator_apply_seconds_bucket{le=...}`
/// rows per `kind` — see [`APPLY_SECONDS_BUCKETS_US`].
const APPLY_SECONDS_BUCKET_COUNT: usize = 12;

/// #4326: `lumen_coordinator_apply_seconds` histogram bucket upper bounds,
/// one entry per `(the Prometheus "le" label, the same bound in whole
/// microseconds)`. Mirrors [`SEARCH_LATENCY_BUCKETS_US`]'s shape and
/// [`Metrics::observe_search`]'s bucket-assignment rule (`<=`, cumulative at
/// render time — see [`Metrics::render_coordinator_apply_histogram`]), sized
/// instead for a single-record apply: sub-millisecond to a 10s outlier.
const APPLY_SECONDS_BUCKETS_US: [(&str, u64); APPLY_SECONDS_BUCKET_COUNT] = [
    ("0.001", 1_000),
    ("0.005", 5_000),
    ("0.01", 10_000),
    ("0.025", 25_000),
    ("0.05", 50_000),
    ("0.1", 100_000),
    ("0.25", 250_000),
    ("0.5", 500_000),
    ("1", 1_000_000),
    ("2.5", 2_500_000),
    ("5", 5_000_000),
    ("10", 10_000_000),
];

/// Number of [`ApplyKind`] variants — the row count of the
/// `lumen_coordinator_apply_seconds{kind=...}` /
/// `lumen_coordinator_apply_items_total{kind=...}` families.
pub const APPLY_KIND_COUNT: usize = 9;

/// #4326: the `kind` label of one applied [`crate::log_entry::RaftLogEntry`]
/// — every variant maps 1:1, see [`ApplyKind::from_entry`]. A plain enum
/// (not the borrowed `RaftLogEntry` itself) so
/// [`Metrics::observe_coordinator_apply`] can index its per-kind atomic
/// arrays without re-matching after the entry has already been consumed by
/// `Engine::apply_prepared_raft_entry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyKind {
    CreateCollection,
    Index,
    ReplaceDocs,
    TruncateDocs,
    UnindexDocs,
    Delete,
    DropCollection,
    AddField,
    DropField,
}

/// One bounded segment of an admitted coordinator request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinatorStage {
    AdmissionToMutationGate,
    PublishToApplyStart,
    ApplyToWaiter,
}

impl CoordinatorStage {
    const ALL: [Self; 3] = [
        Self::AdmissionToMutationGate,
        Self::PublishToApplyStart,
        Self::ApplyToWaiter,
    ];

    const fn index(self) -> usize {
        self as usize
    }

    const fn label(self) -> &'static str {
        match self {
            Self::AdmissionToMutationGate => "admission_to_mutation_gate",
            Self::PublishToApplyStart => "publish_to_apply_start",
            Self::ApplyToWaiter => "apply_to_waiter",
        }
    }
}

impl ApplyKind {
    /// Every kind, in `RaftLogEntry` declaration order. Rendering iterates
    /// this, so the published row set is fixed and a scrape never silently
    /// loses a kind that has not been observed yet.
    pub const ALL: [ApplyKind; APPLY_KIND_COUNT] = [
        ApplyKind::CreateCollection,
        ApplyKind::Index,
        ApplyKind::ReplaceDocs,
        ApplyKind::TruncateDocs,
        ApplyKind::UnindexDocs,
        ApplyKind::Delete,
        ApplyKind::DropCollection,
        ApplyKind::AddField,
        ApplyKind::DropField,
    ];

    /// The `RaftLogEntry` variant name in lowercase snake_case, matching the
    /// wire vocabulary an operator already knows from the API (`index`,
    /// `replace`, `unindex`, ...) rather than the Rust variant spelling.
    pub const fn label(self) -> &'static str {
        match self {
            ApplyKind::CreateCollection => "create_collection",
            ApplyKind::Index => "index",
            ApplyKind::ReplaceDocs => "replace",
            ApplyKind::TruncateDocs => "truncate_docs",
            ApplyKind::UnindexDocs => "unindex",
            ApplyKind::Delete => "delete",
            ApplyKind::DropCollection => "drop_collection",
            ApplyKind::AddField => "add_field",
            ApplyKind::DropField => "drop_field",
        }
    }

    /// This kind's slot in the per-kind atomic arrays.
    const fn index(self) -> usize {
        self as usize
    }

    /// Classify one committed mutation for `lumen_coordinator_apply_seconds`
    /// / `lumen_coordinator_apply_items_total`.
    pub const fn from_entry(entry: &crate::log_entry::RaftLogEntry) -> ApplyKind {
        use crate::log_entry::RaftLogEntry;
        match entry {
            RaftLogEntry::CreateCollection { .. } => ApplyKind::CreateCollection,
            RaftLogEntry::Index { .. } => ApplyKind::Index,
            RaftLogEntry::ReplaceDocs { .. } => ApplyKind::ReplaceDocs,
            RaftLogEntry::TruncateDocs { .. } => ApplyKind::TruncateDocs,
            RaftLogEntry::UnindexDocs { .. } => ApplyKind::UnindexDocs,
            RaftLogEntry::Delete { .. } => ApplyKind::Delete,
            RaftLogEntry::DropCollection { .. } => ApplyKind::DropCollection,
            RaftLogEntry::AddField { .. } => ApplyKind::AddField,
            RaftLogEntry::DropField { .. } => ApplyKind::DropField,
        }
    }
}

/// #4326: the `kind` label for one applied `RaftLogEntry` — see
/// [`ApplyKind::from_entry`]. Kept as a standalone `&'static str` helper
/// (in addition to [`ApplyKind`] itself) for call sites that only need the
/// rendered label, not the enum.
pub fn kind_label(entry: &crate::log_entry::RaftLogEntry) -> &'static str {
    ApplyKind::from_entry(entry).label()
}

/// #4326: items charged to one `lumen_coordinator_apply_items_total{kind}`
/// observation — docs for `index`/`replace`, external ids for `unindex`,
/// `1` for every single-record kind (`create_collection`, `truncate_docs`,
/// `delete`, `drop_collection`, `add_field`, `drop_field`).
pub fn apply_item_count(entry: &crate::log_entry::RaftLogEntry) -> u64 {
    use crate::log_entry::RaftLogEntry;
    match entry {
        RaftLogEntry::Index { req, .. } => req.items.len() as u64,
        RaftLogEntry::ReplaceDocs { req, .. } => req.docs.len() as u64,
        RaftLogEntry::UnindexDocs { req, .. } => req.external_ids.len() as u64,
        _ => 1,
    }
}

/// All metrics carry the `{collection, shard, partition}` label set per
/// the README §5 contract. v1 in-memory single-shard reports
/// `shard="0", partition="0"` as constants; future LSM/Raft tiers will
/// vary `partition` and `shard` respectively.
#[derive(Debug, Default)]
pub struct Metrics {
    pub index_writes_total: Counter,
    pub index_bytes_total: Counter,
    /// Successful immutable Text row preparation in this Engine.
    pub(crate) text_row_stage_rows_total: Counter,
    pub(crate) text_row_stage_input_bytes_total: Counter,
    pub search_requests_total: Counter,
    /// #2519 DEPRECATED: kept only for dashboard back-compat. New
    /// consumers should read the `lumen_search_latency_seconds` histogram
    /// (`search_latency_buckets`/`search_latency_us_sum`) instead — it
    /// carries the same observations at real bucket + second-precision
    /// sum resolution instead of a lossy millisecond-rounded sum/count
    /// pair.
    pub search_latency_ms_sum: Counter,
    /// #2519 DEPRECATED: see `search_latency_ms_sum`. Doubles as the
    /// histogram's total observation count (`+Inf` bucket / `_count`) in
    /// `render_search_latency_histogram` — one `observe_search` call is
    /// exactly one observation of both series, so a second atomic would
    /// only duplicate this one.
    pub search_latency_ms_count: Counter,
    /// #2519: exclusive per-bucket search-latency observation counts
    /// backing `lumen_search_latency_seconds_bucket{le=...}` — see
    /// [`SEARCH_LATENCY_BUCKETS_US`] for the bucket bounds/semantics and
    /// `render_search_latency_histogram` for how these become the
    /// cumulative counts a Prometheus histogram requires.
    pub search_latency_buckets: [Counter; SEARCH_LATENCY_BUCKET_COUNT],
    /// #2519: sum of search latencies in whole microseconds — an integer
    /// atomic (not a float) for lock-free accumulation; `render()` divides
    /// by 1e6 to produce `lumen_search_latency_seconds_sum`.
    pub search_latency_us_sum: Counter,
    /// #2519: total search queries whose latency met or exceeded
    /// `slow_query_threshold_ms` (`LUMEN_SLOW_QUERY_MS`, default
    /// [`DEFAULT_SLOW_QUERY_THRESHOLD_MS`]ms) — see
    /// [`Metrics::observe_search`].
    pub slow_queries_total: Counter,
    /// #2519: the resolved `LUMEN_SLOW_QUERY_MS` threshold in
    /// milliseconds, read once in [`Metrics::new`] (server startup)
    /// rather than per `observe_search` call — mirrors `LUMEN_HNSW_EF`'s
    /// read-once-at-construction convention (`hnsw_search_ef` in
    /// `src/vector_index.rs`) instead of adding env-var lock traffic to
    /// the search hot path. Not published as its own series;
    /// `lumen_slow_queries_total`'s HELP text documents the env var for
    /// operators reading `/metrics` directly.
    pub slow_query_threshold_ms: Gauge,
    pub duplicates_requests_total: Counter,
    pub collections_created_total: Counter,
    pub schema_fields_total: Counter,
    pub storage_bytes: Gauge,
    pub posting_cache_hits_total: Counter,
    pub posting_cache_misses_total: Counter,
    /// #1293: `docs:replace` server-side no-op suppression — fields whose
    /// incoming value matched the currently indexed state and were skipped
    /// (no posting-list rewrite, no HNSW tombstone/reinsert). Distinct from
    /// `index_writes_total`, which only ever counts fields actually written.
    pub replace_fields_skipped_total: Counter,
    /// #1467 R5: this pod's live routed shard-map version, `0` for every
    /// non-routed deployment shape (standalone, primary/replica, no
    /// `operator` feature) that never sets it. The reshard driver's
    /// `advance_convergence` scrapes this over `/metrics` — the same
    /// admin-reachable surface its usage loop already polls — to require
    /// every serving pod to actually report the new map version, not just
    /// that its StatefulSet rollout finished (rollout completion alone does
    /// not prove the ConfigMap write each pod reads its map from has
    /// propagated to every pod).
    pub shard_map_version: Gauge,
    /// #1467 R6: count of scatter (routing-key-less) search sub-requests
    /// where the responding pod's live shard-map version differed from the
    /// scattering pod's own declared version. Signal for a mixed-map
    /// rolling-restart window landing a scatter search mid-flight;
    /// non-fatal by design (see `routing_remote.rs`'s scatter exemption
    /// doc — availability over completeness). `0` outside routed
    /// deployments.
    pub scatter_map_version_mismatches_total: Counter,
    /// #2475: this pod's raft shard index, or the `NOT_RAFT` sentinel until
    /// its raft election-state poller (`spawn_cluster_state_poller` in
    /// `src/bin/lumen.rs`) has ticked at least once. `render()` reads the
    /// sentinel to omit `lumen_raft_leader_known` entirely for
    /// standalone/non-raft deployments — publishing a permanently-`0`
    /// series there would be indistinguishable from a genuinely stuck
    /// leaderless shard to `render::prometheus_rule`'s
    /// `LumenRaftLeaderAbsent` alert.
    pub raft_shard: Gauge,
    /// #2475: `1` while this pod's raft election-state poll believes its
    /// shard currently has an elected leader (itself or a peer), `0` while
    /// it does not. Meaningful only once `raft_shard` has left the
    /// `NOT_RAFT` sentinel; see [`Metrics::set_raft_leader_known`].
    pub raft_leader_known: Gauge,
    /// #2475: `1` while this pod believes a reshard-driver write fence
    /// (`POST /admin/reshard:fence`, see `crate::api::WriteFence`) is
    /// currently armed on it, `0` once cleared or never armed.
    pub reshard_fence_active: Gauge,
    /// #2475: unix-epoch seconds the currently (or most recently) armed
    /// write fence was armed at. Pairs with `reshard_fence_active` so
    /// `LumenReshardWorkflowStalled` can tell a long-armed fence (the
    /// driver's fenced final `CatchingUp` pass never came back to clear it)
    /// from one still mid-pass.
    pub reshard_fence_armed_unixtime: Gauge,
    /// #2516: `1` while this node is in ENOSPC degraded read-only mode
    /// (a durable write path — local AOF/WAL append, segment/RDB
    /// checkpoint save, or raft log append — hit
    /// `io::ErrorKind::StorageFull`), `0` otherwise. Sticky: stays `1`
    /// until the periodic re-probe (`LUMEN_STORAGE_FULL_REPROBE_SECS`, see
    /// `src/bin/lumen.rs`) confirms the data dir accepts a write again, or
    /// the process restarts. `crate::api::enforce_storage_writable` reads
    /// this to fast-fail mutating endpoints without touching the durable
    /// path; `render::prometheus_rule`'s `LumenStorageDegraded` alert reads
    /// the published `lumen_storage_degraded` series.
    pub storage_degraded: Gauge,
    /// #2516: total genuine ENOSPC hits observed on a durable write path,
    /// monotonic across the process lifetime (never reset when
    /// `storage_degraded` clears) — see [`Metrics::mark_storage_degraded`].
    pub storage_full_errors_total: Counter,
    /// Completed durable segment checkpoints. This remains zero until the
    /// segment checkpoint path calls [`Metrics::observe_segment_checkpoint`].
    pub segment_checkpoint_completed_total: Counter,
    /// Checkpoint attempts that entered the shared synchronous checkpoint
    /// wrapper, including attempts that later fail before publication.
    pub segment_checkpoint_started_total: Counter,
    /// Checkpoint attempts that returned an error from the shared synchronous
    /// checkpoint wrapper.
    pub segment_checkpoint_failed_total: Counter,
    /// Checkpoint attempts currently inside the shared synchronous wrapper.
    pub segment_checkpoint_in_flight: Gauge,
    /// Actual bytes durably published by completed segment checkpoints.
    pub segment_checkpoint_bytes_total: Counter,
    /// Sum of completed segment checkpoint durations in microseconds. Rendered
    /// as seconds by `lumen_segment_checkpoint_duration_seconds_sum`.
    pub segment_checkpoint_duration_us_sum: Counter,
    /// Number of completed segment checkpoint duration observations.
    pub segment_checkpoint_duration_count: Counter,
    /// Sum of capture-lock hold durations in microseconds. Rendered as seconds
    /// by `lumen_segment_capture_lock_seconds_sum`.
    pub segment_capture_lock_us_sum: Counter,
    /// Number of capture-lock hold duration observations.
    pub segment_capture_lock_count: Counter,
    /// Completed durable segment merges. This remains zero until the segment
    /// merge path calls [`Metrics::observe_segment_merge`].
    pub segment_merge_completed_total: Counter,
    /// Bytes read while completed durable segment merges run.
    pub segment_merge_read_bytes_total: Counter,
    /// Bytes written while completed durable segment merges run.
    pub segment_merge_write_bytes_total: Counter,
    /// Per-[`MergeStep`] sum of step durations in microseconds, rendered as
    /// seconds by `lumen_segment_merge_phase_seconds_sum{phase=...}`.
    pub segment_merge_step_us_sum: [Counter; MERGE_STEP_COUNT],
    /// Per-[`MergeStep`] observation count, rendered as
    /// `lumen_segment_merge_phase_seconds_count{phase=...}`.
    pub segment_merge_step_count: [Counter; MERGE_STEP_COUNT],
    /// Per-[`MergeStep`] count of filesystem entries the step touched —
    /// hard-links created, files inherited, files validated. Zero for a step
    /// whose cost is not file-count-shaped, which is exactly the signal that
    /// separates "the root is wide" from "the payload is large".
    pub segment_merge_step_files: [Counter; MERGE_STEP_COUNT],
    /// Total files hard-linked by completed merge jobs across both
    /// whole-root link passes (`link_scratch` + `link_generation`).
    pub segment_merge_linked_files_total: Counter,
    /// Total fields compacted by merge jobs. This is a per-field count,
    /// unlike `segment_merge_completed_total`, which one job increments once
    /// per published output.
    pub segment_merge_fields_total: Counter,
    /// Sum of the whole publication-side `save_gate` hold in microseconds,
    /// rendered as `lumen_segment_merge_save_gate_held_seconds_sum`.
    pub segment_merge_save_gate_us_sum: Counter,
    /// Observation count for `segment_merge_save_gate_us_sum`.
    pub segment_merge_save_gate_count: Counter,
    /// Bytes that remain in unmerged segment delta layers.
    pub segment_pending_delta_bytes: Gauge,
    /// Number of unmerged segment delta layers.
    pub segment_pending_delta_layers: Gauge,
    /// Total admissions delayed or refused due to segment backpressure.
    pub segment_backpressure_total: Counter,
    /// Actual durable segment files on disk in bytes.
    pub segment_disk_bytes: Gauge,
    /// #4326: per-[`ApplyKind`] exclusive per-bucket observation counts
    /// backing `lumen_coordinator_apply_seconds_bucket{le=...,kind=...}` —
    /// see [`APPLY_SECONDS_BUCKETS_US`] for the bucket bounds and
    /// [`Metrics::render_coordinator_apply_histogram`] for how these become
    /// the cumulative counts a Prometheus histogram requires.
    pub coordinator_apply_seconds_buckets:
        [[Counter; APPLY_SECONDS_BUCKET_COUNT]; APPLY_KIND_COUNT],
    /// #4326: per-[`ApplyKind`] sum of write-coordinator per-record apply
    /// durations in whole microseconds — an integer atomic for lock-free
    /// accumulation; `render()` divides by 1e6 to produce
    /// `lumen_coordinator_apply_seconds_sum{kind=...}`.
    pub coordinator_apply_seconds_us_sum: [Counter; APPLY_KIND_COUNT],
    /// #4326: per-[`ApplyKind`] total apply observations, backing
    /// `lumen_coordinator_apply_seconds_count{kind=...}` and the `+Inf`
    /// bucket.
    pub coordinator_apply_seconds_count: [Counter; APPLY_KIND_COUNT],
    /// #4326: per-[`ApplyKind`] total items applied (docs for
    /// `index`/`replace`, external ids for `unindex`, `1` otherwise),
    /// backing `lumen_coordinator_apply_items_total{kind=...}` — divide by
    /// `coordinator_apply_seconds_count`'s sibling sum for per-item apply
    /// cost.
    pub coordinator_apply_items_total: [Counter; APPLY_KIND_COUNT],
    /// #4246: bounded request-stage histograms, labelled by operation kind.
    pub coordinator_stage_seconds_buckets:
        [[[Counter; APPLY_SECONDS_BUCKET_COUNT]; APPLY_KIND_COUNT]; 3],
    pub coordinator_stage_seconds_us_sum: [[Counter; APPLY_KIND_COUNT]; 3],
    pub coordinator_stage_seconds_count: [[Counter; APPLY_KIND_COUNT]; 3],
    /// Exclusive finite-bucket observations for time spent waiting to acquire
    /// the committed-apply engine state write lock.
    pub engine_state_write_lock_wait_seconds_buckets: [Counter; APPLY_SECONDS_BUCKET_COUNT],
    pub engine_state_write_lock_wait_seconds_us_sum: Counter,
    pub engine_state_write_lock_wait_seconds_count: Counter,
    /// Exclusive finite-bucket observations for time the committed-apply
    /// engine state write lock is held. Published after the guard drops.
    pub engine_state_write_lock_held_seconds_buckets: [Counter; APPLY_SECONDS_BUCKET_COUNT],
    pub engine_state_write_lock_held_seconds_us_sum: Counter,
    pub engine_state_write_lock_held_seconds_count: Counter,
    /// Exclusive finite-bucket observations for live HNSW graph additions in
    /// committed apply. Flat CPU additions do not record this family.
    pub hnsw_add_seconds_buckets: [Counter; APPLY_SECONDS_BUCKET_COUNT],
    pub hnsw_add_seconds_us_sum: Counter,
    pub hnsw_add_seconds_count: Counter,
    /// HNSW graph rebuild time. Normal additions do not record this family.
    pub hnsw_graph_rebuild_seconds_buckets: [Counter; APPLY_SECONDS_BUCKET_COUNT],
    pub hnsw_graph_rebuild_seconds_us_sum: Counter,
    pub hnsw_graph_rebuild_seconds_count: Counter,
    /// HNSW graph write-lock acquisition time, excluding the work while held.
    pub hnsw_write_lock_wait_seconds_buckets: [Counter; APPLY_SECONDS_BUCKET_COUNT],
    pub hnsw_write_lock_wait_seconds_us_sum: Counter,
    pub hnsw_write_lock_wait_seconds_count: Counter,
    /// HNSW graph write-lock hold time, excluding the time waiting to acquire it.
    pub hnsw_write_lock_held_seconds_buckets: [Counter; APPLY_SECONDS_BUCKET_COUNT],
    pub hnsw_write_lock_held_seconds_us_sum: Counter,
    pub hnsw_write_lock_held_seconds_count: Counter,
    /// Completed capacity-relief cycles that ran checkpoint then merge.
    pub segment_capacity_relief_checkpoint_merge_seconds_buckets:
        [Counter; APPLY_SECONDS_BUCKET_COUNT],
    pub segment_capacity_relief_checkpoint_merge_seconds_us_sum: Counter,
    pub segment_capacity_relief_checkpoint_merge_seconds_count: Counter,
    /// Linux `/proc/self/status` `VmHWM` in bytes at the latest scrape. This
    /// is meaningful only while `process_rss_high_water_available` is `1`.
    pub process_rss_high_water_bytes: Gauge,
    /// `1` when this scrape parsed Linux `VmHWM`; `0` when this process cannot
    /// report it. Consumers must require this signal before accepting the RSS
    /// value, so an unavailable platform cannot qualify with a zero gauge.
    pub process_rss_high_water_available: Gauge,
    /// Process-wide bytes reserved before apply for pending durable changes.
    pub pending_change_reserved_bytes: Gauge,
    /// Process-wide active bytes owned by pending durable changes.
    pub pending_change_active_bytes: Gauge,
    /// Process-wide frozen bytes retained by incomplete durable checkpoints.
    pub pending_change_frozen_bytes: Gauge,
    /// Process-wide pending durable-change bytes across all ownership states.
    pub pending_change_total_bytes: Gauge,
    /// Largest observed process-wide pending durable-change total. This stays
    /// monotonic for the process lifetime, including peaks between scrapes.
    pub pending_change_high_water_bytes: Gauge,
}

#[derive(Clone, Copy)]
struct PendingChangeAccounting {
    reserved: u64,
    active: u64,
    frozen: u64,
    total: u64,
    high_water: u64,
}

/// One state-writing apply telemetry scope. It stores measurements locally while
/// the engine state write lock is held, then publishes them only after that
/// guard drops. This keeps metrics atomics out of the state-lock interval.
pub(crate) struct CommittedApplyTelemetry<'a> {
    metrics: &'a Metrics,
    state_write_lock_wait: Option<Duration>,
    state_write_lock_held_started: Option<std::time::Instant>,
    hnsw_adds: ApplyDurationObservations,
    hnsw_graph_rebuilds: ApplyDurationObservations,
    hnsw_write_lock_waits: ApplyDurationObservations,
    hnsw_write_lock_holds: ApplyDurationObservations,
}

impl CommittedApplyTelemetry<'_> {
    /// Store the state-write wait without touching global metrics.
    pub(crate) fn record_state_write_lock_wait(&mut self, elapsed: Duration) {
        debug_assert!(self.state_write_lock_wait.is_none());
        self.state_write_lock_wait = Some(elapsed);
    }

    /// Begin the engine writer hold interval after the guard is acquired.
    pub(crate) fn start_state_write_lock_hold(&mut self) {
        debug_assert!(self.state_write_lock_held_started.is_none());
        self.state_write_lock_held_started = Some(std::time::Instant::now());
    }

    /// Finish the writer interval immediately after dropping the state guard.
    /// On an early return, [`Drop`] runs after that guard and finishes it.
    pub(crate) fn finish_state_write_lock_hold(&mut self) {
        if let Some(started) = self.state_write_lock_held_started.take() {
            self.metrics
                .observe_engine_state_write_lock_hold(started.elapsed());
        }
    }

    /// Record one HNSW graph-add duration without touching global metrics.
    pub(crate) fn record_hnsw_add(&mut self, elapsed: Duration) {
        self.hnsw_adds.record(elapsed);
    }

    /// Store a rare HNSW rebuild interval without publishing atomics under the
    /// Engine writer lock.
    pub(crate) fn record_hnsw_graph_rebuild(&mut self, elapsed: Duration) {
        self.hnsw_graph_rebuilds.record(elapsed);
    }

    /// Store an HNSW lock split without publishing atomics under Engine lock.
    pub(crate) fn record_hnsw_write_lock(&mut self, wait: Duration, held: Duration) {
        self.hnsw_write_lock_waits.record(wait);
        self.hnsw_write_lock_holds.record(held);
    }
}

impl Drop for CommittedApplyTelemetry<'_> {
    fn drop(&mut self) {
        if let Some(elapsed) = self.state_write_lock_wait {
            self.metrics.observe_engine_state_write_lock_wait(elapsed);
        }
        self.finish_state_write_lock_hold();
        self.metrics.observe_hnsw_add_observations(&self.hnsw_adds);
        self.metrics
            .observe_hnsw_graph_rebuild_observations(&self.hnsw_graph_rebuilds);
        self.metrics
            .observe_hnsw_write_lock_wait_observations(&self.hnsw_write_lock_waits);
        self.metrics
            .observe_hnsw_write_lock_hold_observations(&self.hnsw_write_lock_holds);
    }
}

/// Stack-local histogram observations for one committed-apply scope.
#[derive(Default)]
struct ApplyDurationObservations {
    buckets: [u64; APPLY_SECONDS_BUCKET_COUNT],
    micros_sum: u64,
    count: u64,
}

impl ApplyDurationObservations {
    fn record(&mut self, elapsed: Duration) {
        let micros = duration_to_micros(elapsed);
        if let Some(bucket_idx) = APPLY_SECONDS_BUCKETS_US
            .iter()
            .position(|&(_, bound_us)| micros <= bound_us)
        {
            self.buckets[bucket_idx] = self.buckets[bucket_idx].wrapping_add(1);
        }
        self.micros_sum = self.micros_sum.wrapping_add(micros);
        self.count = self.count.wrapping_add(1);
    }
}

impl Metrics {
    pub fn new() -> Self {
        let metrics = Self::default();
        // #2475: seed the "never touched by the raft poller" sentinel;
        // every other field's `Default` (0) is already the right initial
        // value.
        metrics.raft_shard.set(NOT_RAFT);
        // #2519: resolve `LUMEN_SLOW_QUERY_MS` once at construction time.
        metrics
            .slow_query_threshold_ms
            .set(slow_query_threshold_ms_from_env());
        metrics
    }

    pub fn incr_index(&self, items: u64, bytes: u64) {
        self.index_writes_total.add(items);
        self.index_bytes_total.add(bytes);
    }

    /// Record one successfully completed durable segment checkpoint. The
    /// checkpoint implementation must call this only after its durable commit
    /// succeeds. `checkpoint_bytes` is the actual durable segment output;
    /// `elapsed` is the complete operation duration; `capture_lock_hold` is
    /// only the interval in which the capture lock blocks concurrent writers.
    pub fn observe_segment_checkpoint(
        &self,
        checkpoint_bytes: u64,
        elapsed: Duration,
        capture_lock_hold: Duration,
    ) {
        self.segment_checkpoint_completed_total.incr();
        self.segment_checkpoint_bytes_total.add(checkpoint_bytes);
        let elapsed_us = duration_to_micros(elapsed);
        self.segment_checkpoint_duration_us_sum.add(elapsed_us);
        self.segment_checkpoint_duration_count.incr();
        let hold_us = duration_to_micros(capture_lock_hold);
        self.segment_capture_lock_us_sum.add(hold_us);
        self.segment_capture_lock_count.incr();
    }

    /// Mark one shared synchronous checkpoint attempt as started. The caller
    /// must pair this with [`Metrics::finish_segment_checkpoint_attempt`].
    pub fn start_segment_checkpoint_attempt(&self) {
        self.segment_checkpoint_started_total.incr();
        self.segment_checkpoint_in_flight
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Mark one shared synchronous checkpoint attempt as finished. Failed
    /// attempts remain distinct from durable completions and their bytes.
    pub fn finish_segment_checkpoint_attempt(&self, failed: bool) {
        if failed {
            self.segment_checkpoint_failed_total.incr();
        }
        self.segment_checkpoint_in_flight
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record one successfully completed durable segment merge. The merge
    /// implementation must call this only after its output is durable and
    /// visible. `read_bytes` and `write_bytes` are actual merge I/O.
    pub fn observe_segment_merge(&self, read_bytes: u64, write_bytes: u64) {
        self.segment_merge_completed_total.incr();
        self.segment_merge_read_bytes_total.add(read_bytes);
        self.segment_merge_write_bytes_total.add(write_bytes);
    }

    /// Record one timed `step` of a background merge job. `files` is the
    /// number of filesystem entries the step touched, or `0` for a step whose
    /// cost is not file-shaped. Called a handful of times per merge job and
    /// never from a search path.
    ///
    /// The two whole-root hard-link steps also accumulate
    /// `segment_merge_linked_files_total`, so a scrape can compare "files
    /// linked" against "fields compacted" without summing label rows.
    pub fn observe_segment_merge_step(&self, step: MergeStep, elapsed: Duration, files: u64) {
        let index = step.index();
        self.segment_merge_step_us_sum[index].add(duration_to_micros(elapsed));
        self.segment_merge_step_count[index].incr();
        self.segment_merge_step_files[index].add(files);
        if matches!(step, MergeStep::LinkScratch | MergeStep::LinkGeneration) {
            self.segment_merge_linked_files_total.add(files);
        }
    }

    /// Record the whole publication-side `save_gate` hold of one merge job.
    pub fn observe_segment_merge_save_gate(&self, held: Duration) {
        self.segment_merge_save_gate_us_sum
            .add(duration_to_micros(held));
        self.segment_merge_save_gate_count.incr();
    }

    /// Record that one merge job compacted `fields` fields.
    pub fn incr_segment_merge_fields(&self, fields: u64) {
        self.segment_merge_fields_total.add(fields);
    }

    /// Read back one step's `(microsecond sum, observation count, files)`.
    /// This is the same registry production scrapes, so a measurement that
    /// reads it cannot drift from what `/metrics` publishes.
    pub fn segment_merge_step_observation(&self, step: MergeStep) -> (u64, u64, u64) {
        let index = step.index();
        (
            self.segment_merge_step_us_sum[index].get(),
            self.segment_merge_step_count[index].get(),
            self.segment_merge_step_files[index].get(),
        )
    }

    /// Set the current unmerged delta backlog from the segment state.
    pub fn set_segment_pending_delta(&self, bytes: u64, layers: u64) {
        self.segment_pending_delta_bytes.set(bytes);
        self.segment_pending_delta_layers.set(layers);
    }

    /// Record one admission delayed or refused by real segment backpressure.
    pub fn incr_segment_backpressure(&self) {
        self.segment_backpressure_total.incr();
    }

    /// #4326: record one write-coordinator apply of `items` items of `kind`
    /// taking `elapsed` — call once per admitted local record, timed from
    /// just before `prepare_local_record` (which covers any reprice/
    /// `wait_grow_to` retry) to just after `Engine::apply_prepared_raft_entry`
    /// returns. `seconds_sum / items_total` is the per-item apply cost a
    /// large-scale perf probe reads from `GET /metrics`.
    pub fn observe_coordinator_apply(&self, kind: ApplyKind, items: u64, elapsed: Duration) {
        let idx = kind.index();
        let us = duration_to_micros(elapsed);
        if let Some(bucket_idx) = APPLY_SECONDS_BUCKETS_US
            .iter()
            .position(|&(_, bound_us)| us <= bound_us)
        {
            self.coordinator_apply_seconds_buckets[idx][bucket_idx].incr();
        }
        self.coordinator_apply_seconds_us_sum[idx].add(us);
        self.coordinator_apply_seconds_count[idx].incr();
        self.coordinator_apply_items_total[idx].add(items);
    }

    /// Record one coordinator stage without changing request scheduling.
    pub fn observe_coordinator_stage(
        &self,
        kind: ApplyKind,
        stage: CoordinatorStage,
        elapsed: Duration,
    ) {
        let stage_idx = stage.index();
        let kind_idx = kind.index();
        let us = duration_to_micros(elapsed);
        if let Some(bucket_idx) = APPLY_SECONDS_BUCKETS_US
            .iter()
            .position(|&(_, bound_us)| us <= bound_us)
        {
            self.coordinator_stage_seconds_buckets[stage_idx][kind_idx][bucket_idx].incr();
        }
        self.coordinator_stage_seconds_us_sum[stage_idx][kind_idx].add(us);
        self.coordinator_stage_seconds_count[stage_idx][kind_idx].incr();
    }

    /// Record only the wait to acquire the committed-apply engine state write
    /// lock. This is separate from work performed while the lock is held.
    pub fn observe_engine_state_write_lock_wait(&self, elapsed: Duration) {
        observe_apply_duration_histogram(
            &self.engine_state_write_lock_wait_seconds_buckets,
            &self.engine_state_write_lock_wait_seconds_us_sum,
            &self.engine_state_write_lock_wait_seconds_count,
            elapsed,
        );
    }

    /// Record the engine writer interval only after the state guard drops.
    pub fn observe_engine_state_write_lock_hold(&self, elapsed: Duration) {
        observe_apply_duration_histogram(
            &self.engine_state_write_lock_held_seconds_buckets,
            &self.engine_state_write_lock_held_seconds_us_sum,
            &self.engine_state_write_lock_held_seconds_count,
            elapsed,
        );
    }

    /// Record only one live HNSW graph-add call in committed apply.
    pub fn observe_hnsw_add(&self, elapsed: Duration) {
        let mut observations = ApplyDurationObservations::default();
        observations.record(elapsed);
        self.observe_hnsw_add_observations(&observations);
    }

    /// Record one HNSW graph rebuild apart from the containing add call.
    pub fn observe_hnsw_graph_rebuild(&self, elapsed: Duration) {
        let mut observations = ApplyDurationObservations::default();
        observations.record(elapsed);
        self.observe_hnsw_graph_rebuild_observations(&observations);
    }

    /// Record only the wait to acquire the HNSW graph write lock.
    pub fn observe_hnsw_write_lock_wait(&self, elapsed: Duration) {
        observe_apply_duration_histogram(
            &self.hnsw_write_lock_wait_seconds_buckets,
            &self.hnsw_write_lock_wait_seconds_us_sum,
            &self.hnsw_write_lock_wait_seconds_count,
            elapsed,
        );
    }

    /// Record only the interval the HNSW graph write lock is held.
    pub fn observe_hnsw_write_lock_hold(&self, elapsed: Duration) {
        observe_apply_duration_histogram(
            &self.hnsw_write_lock_held_seconds_buckets,
            &self.hnsw_write_lock_held_seconds_us_sum,
            &self.hnsw_write_lock_held_seconds_count,
            elapsed,
        );
    }

    /// Record a successful capacity-relief checkpoint-plus-merge cycle.
    pub fn observe_capacity_relief_checkpoint_merge_cycle(&self, elapsed: Duration) {
        observe_apply_duration_histogram(
            &self.segment_capacity_relief_checkpoint_merge_seconds_buckets,
            &self.segment_capacity_relief_checkpoint_merge_seconds_us_sum,
            &self.segment_capacity_relief_checkpoint_merge_seconds_count,
            elapsed,
        );
    }

    /// Start a state-writing apply telemetry scope. Its [`Drop`] publishes only
    /// after the state write guard declared after it has dropped.
    pub(crate) fn apply_telemetry(&self) -> CommittedApplyTelemetry<'_> {
        CommittedApplyTelemetry {
            metrics: self,
            state_write_lock_wait: None,
            state_write_lock_held_started: None,
            hnsw_adds: ApplyDurationObservations::default(),
            hnsw_graph_rebuilds: ApplyDurationObservations::default(),
            hnsw_write_lock_waits: ApplyDurationObservations::default(),
            hnsw_write_lock_holds: ApplyDurationObservations::default(),
        }
    }

    /// Start a committed-apply telemetry scope.
    pub(crate) fn committed_apply_telemetry(&self) -> CommittedApplyTelemetry<'_> {
        self.apply_telemetry()
    }

    fn observe_hnsw_add_observations(&self, observations: &ApplyDurationObservations) {
        observe_apply_duration_observations(
            &self.hnsw_add_seconds_buckets,
            &self.hnsw_add_seconds_us_sum,
            &self.hnsw_add_seconds_count,
            observations,
        );
    }

    fn observe_hnsw_write_lock_wait_observations(&self, observations: &ApplyDurationObservations) {
        observe_apply_duration_observations(
            &self.hnsw_write_lock_wait_seconds_buckets,
            &self.hnsw_write_lock_wait_seconds_us_sum,
            &self.hnsw_write_lock_wait_seconds_count,
            observations,
        );
    }

    fn observe_hnsw_graph_rebuild_observations(&self, observations: &ApplyDurationObservations) {
        observe_apply_duration_observations(
            &self.hnsw_graph_rebuild_seconds_buckets,
            &self.hnsw_graph_rebuild_seconds_us_sum,
            &self.hnsw_graph_rebuild_seconds_count,
            observations,
        );
    }

    fn observe_hnsw_write_lock_hold_observations(&self, observations: &ApplyDurationObservations) {
        observe_apply_duration_observations(
            &self.hnsw_write_lock_held_seconds_buckets,
            &self.hnsw_write_lock_held_seconds_us_sum,
            &self.hnsw_write_lock_held_seconds_count,
            observations,
        );
    }

    /// Set the byte size of the current durable segment files on disk.
    pub fn set_segment_disk_bytes(&self, bytes: u64) {
        self.segment_disk_bytes.set(bytes);
    }

    /// Record one search observation of `elapsed`. Updates the deprecated
    /// millisecond sum/count pair (back-compat, see `search_latency_ms_sum`),
    /// the `lumen_search_latency_seconds` histogram (#2519), and
    /// `slow_queries_total` when `elapsed` meets or exceeds
    /// `slow_query_threshold_ms` — `>=`, not `>`, so a
    /// `LUMEN_SLOW_QUERY_MS=0` override (used by tests to force every
    /// search to count as slow) actually fires on a `0`ms observation.
    pub fn observe_search(&self, elapsed: Duration) {
        self.search_requests_total.incr();

        let latency_ms = elapsed.as_millis() as u64;
        self.search_latency_ms_sum.add(latency_ms);
        self.search_latency_ms_count.incr();

        let latency_us = u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
        self.search_latency_us_sum.add(latency_us);
        if let Some(idx) = SEARCH_LATENCY_BUCKETS_US
            .iter()
            .position(|&(_, bound_us)| latency_us <= bound_us)
        {
            self.search_latency_buckets[idx].incr();
        }

        if latency_ms >= self.slow_query_threshold_ms.get() {
            self.slow_queries_total.incr();
        }
    }

    pub fn incr_duplicates(&self) {
        self.duplicates_requests_total.incr();
    }

    /// #1293: record `n` `docs:replace` fields skipped as unchanged no-ops.
    pub fn incr_replace_skipped(&self, fields: u64) {
        self.replace_fields_skipped_total.add(fields);
    }

    pub fn incr_collection_created(&self, fields: u64) {
        self.collections_created_total.incr();
        self.schema_fields_total.add(fields);
    }

    pub fn set_storage_bytes(&self, bytes: u64) {
        self.storage_bytes.set(bytes);
    }

    /// #1467 R5: record this pod's live routed shard-map version.
    pub fn set_shard_map_version(&self, version: u64) {
        self.shard_map_version.set(version);
    }

    /// #1467 R6: record one scatter sub-response whose responding pod's map
    /// version differed from the scattering pod's own declared version.
    pub fn incr_scatter_map_version_mismatch(&self) {
        self.scatter_map_version_mismatches_total.incr();
    }

    /// #2475: record this pod's shard index + whether its raft
    /// election-state poll currently believes that shard has an elected
    /// leader. Called every `spawn_cluster_state_poller` tick in raft mode
    /// only; standalone/non-raft pods never call this, so `raft_shard`
    /// stays at the `NOT_RAFT` sentinel forever and `render()` omits
    /// `lumen_raft_leader_known` for them.
    pub fn set_raft_leader_known(&self, shard: u32, known: bool) {
        self.raft_shard.set(shard as u64);
        self.raft_leader_known.set(known as u64);
    }

    /// #2475: arm/clear the reshard write-fence-active signal, called from
    /// `POST /admin/reshard:fence`'s handler. Arming also stamps
    /// `reshard_fence_armed_unixtime` to "now".
    pub fn set_reshard_fence_active(&self, active: bool) {
        self.reshard_fence_active.set(active as u64);
        if active {
            self.reshard_fence_armed_unixtime.set(unix_now_secs());
        }
    }

    /// #2516: flip into ENOSPC degraded read-only mode and count the hit.
    /// Called from every durable-write-path origin that classifies its
    /// failure as `io::ErrorKind::StorageFull` (see
    /// `crate::coordinator::is_storage_full`) — the coordinator apply
    /// loop's local AOF persist, the periodic RDB/segment checkpoint
    /// snapshotters, and (when the `raft-wal` feature is active) raft log
    /// append. Idempotent: calling it while already degraded still counts
    /// the new hit but leaves the gauge at `1`.
    pub fn mark_storage_degraded(&self) {
        self.storage_full_errors_total.incr();
        self.storage_degraded.set(1);
    }

    /// #2516: `true` while this node is in ENOSPC degraded read-only mode.
    pub fn is_storage_degraded(&self) -> bool {
        self.storage_degraded.get() != 0
    }

    /// #2516: clear degraded mode once the periodic re-probe confirms the
    /// data dir accepts a write again.
    pub fn clear_storage_degraded(&self) {
        self.storage_degraded.set(0);
    }

    /// Refresh the Linux process peak resident set size from `/proc`. Returns
    /// `true` only when the current scrape parsed `VmHWM`; a caller can use
    /// the rendered availability gauge to reject unavailable values.
    pub fn refresh_process_rss_high_water(&self) -> bool {
        match process_rss_high_water_bytes() {
            Some(bytes) => {
                self.process_rss_high_water_bytes.set(bytes);
                self.process_rss_high_water_available.set(1);
                true
            }
            None => {
                self.process_rss_high_water_bytes.set(0);
                self.process_rss_high_water_available.set(0);
                false
            }
        }
    }

    /// Refresh process-wide pending-change accounting from the shared budget.
    /// The high-water gauge is monotonic in the budget and therefore retains
    /// increases that happened between Prometheus scrapes.
    pub fn refresh_pending_change_accounting(&self) {
        let budget = ChangeBudget::process_shared();
        self.read_pending_change_accounting(&budget);
    }

    /// Read one coherent budget state and mirror it into the compatibility
    /// gauges. Render uses the returned local values, never five later atomic
    /// reads that another concurrent render could mix.
    fn read_pending_change_accounting(&self, budget: &ChangeBudget) -> PendingChangeAccounting {
        let (snapshot, high_water) = budget.snapshot_with_high_water();
        let values = PendingChangeAccounting {
            reserved: snapshot.reserved as u64,
            active: snapshot.active as u64,
            frozen: snapshot.frozen as u64,
            total: snapshot.total as u64,
            high_water: high_water as u64,
        };
        self.pending_change_reserved_bytes.set(values.reserved);
        self.pending_change_active_bytes.set(values.active);
        self.pending_change_frozen_bytes.set(values.frozen);
        self.pending_change_total_bytes.set(values.total);
        self.pending_change_high_water_bytes.set(values.high_water);
        values
    }

    /// Prometheus text format (0.0.4 compatible). Always emits the same
    /// set of metric names so scrape configs are stable.
    pub fn render(&self) -> String {
        self.refresh_process_rss_high_water();
        let budget = ChangeBudget::process_shared();
        let pending_changes = self.read_pending_change_accounting(&budget);
        let samples = [
            Sample::new(
                "lumen_index_writes_total",
                "counter",
                "Total index items applied.",
                self.index_writes_total.get(),
            ),
            Sample::new(
                "lumen_index_bytes_total",
                "counter",
                "Total bytes written across all field indexes.",
                self.index_bytes_total.get(),
            ),
            Sample::new(
                "lumen_search_requests_total",
                "counter",
                "Total search requests served.",
                self.search_requests_total.get(),
            ),
            Sample::new(
                "lumen_search_latency_ms_sum",
                "counter",
                "DEPRECATED (#2519): sum of search latencies in milliseconds; kept for \
                 dashboard back-compat, see lumen_search_latency_seconds for the real \
                 histogram.",
                self.search_latency_ms_sum.get(),
            ),
            Sample::new(
                "lumen_search_latency_ms_count",
                "counter",
                "DEPRECATED (#2519): count of search latency observations; kept for \
                 dashboard back-compat, see lumen_search_latency_seconds for the real \
                 histogram.",
                self.search_latency_ms_count.get(),
            ),
            Sample::new(
                "lumen_slow_queries_total",
                "counter",
                "Total search queries at/above the slow-query threshold \
                 (LUMEN_SLOW_QUERY_MS, default 500ms).",
                self.slow_queries_total.get(),
            ),
            Sample::new(
                "lumen_duplicates_requests_total",
                "counter",
                "Total duplicate-detection requests.",
                self.duplicates_requests_total.get(),
            ),
            Sample::new(
                "lumen_collections_created_total",
                "counter",
                "Total collections created or extended.",
                self.collections_created_total.get(),
            ),
            Sample::new(
                "lumen_schema_fields_total",
                "counter",
                "Total field declarations registered.",
                self.schema_fields_total.get(),
            ),
            Sample::new(
                "lumen_storage_bytes",
                "gauge",
                "Approximate bytes held by all in-memory field indexes.",
                self.storage_bytes.get(),
            ),
            Sample::new(
                "lumen_posting_cache_hits_total",
                "counter",
                "Posting cache hit count (0 until LSM cache is wired).",
                self.posting_cache_hits_total.get(),
            ),
            Sample::new(
                "lumen_posting_cache_misses_total",
                "counter",
                "Posting cache miss count.",
                self.posting_cache_misses_total.get(),
            ),
            Sample::new(
                "lumen_replace_fields_skipped_total",
                "counter",
                "Total docs:replace fields skipped as unchanged no-ops.",
                self.replace_fields_skipped_total.get(),
            ),
            Sample::new(
                "lumen_shard_map_version",
                "gauge",
                "This pod's live routed shard-map version (0 outside routed deployments).",
                self.shard_map_version.get(),
            ),
            Sample::new(
                "lumen_scatter_map_version_mismatches_total",
                "counter",
                "Scatter search sub-responses whose responding pod's map version differed \
                 from the sender's.",
                self.scatter_map_version_mismatches_total.get(),
            ),
            Sample::new(
                "lumen_reshard_fence_active",
                "gauge",
                "1 while this pod believes a reshard-driver write fence is currently armed \
                 on it.",
                self.reshard_fence_active.get(),
            ),
            Sample::new(
                "lumen_reshard_fence_armed_unixtime",
                "gauge",
                "Unix time the currently (or most recently) armed reshard write fence was \
                 armed at.",
                self.reshard_fence_armed_unixtime.get(),
            ),
            Sample::new(
                "lumen_storage_degraded",
                "gauge",
                "1 while this node is in ENOSPC degraded read-only mode (a durable write \
                 path hit disk-full).",
                self.storage_degraded.get(),
            ),
            Sample::new(
                "lumen_storage_full_errors_total",
                "counter",
                "Total genuine ENOSPC hits observed on a durable write path.",
                self.storage_full_errors_total.get(),
            ),
        ];
        let mut out = metrics_prometheus::render(&samples);
        // #2519: real Prometheus histogram for search latency — hand-rolled
        // rather than via `metrics_prometheus::render_labeled` because a
        // histogram's `_bucket`/`_sum`/`_count` sample names all suffix ONE
        // base name under a single `# HELP`/`# TYPE histogram` pair, while
        // `render_labeled` assumes one bare metric name per row.
        out.push_str(&self.render_search_latency_histogram());
        // #2475: `lumen_raft_leader_known` carries a `shard` label and is
        // omitted entirely (not just left at 0) for standalone/non-raft
        // pods — see `raft_shard`'s doc comment for why a permanent-0
        // series would be a false-positive risk for `LumenRaftLeaderAbsent`.
        let shard = self.raft_shard.get();
        if shard != NOT_RAFT {
            let shard_label = shard.to_string();
            let rows = [LabeledSample::new(
                vec![Label::new("shard", &shard_label)],
                self.raft_leader_known.get(),
            )];
            let groups = [SampleGroup::new(
                "lumen_raft_leader_known",
                "gauge",
                "1 while this pod's raft election-state poll believes its shard currently \
                 has an elected leader, 0 otherwise. Omitted for standalone/non-raft \
                 deployments.",
                &rows,
            )];
            out.push_str(&metrics_prometheus::render_labeled(&groups));
        }
        out.push_str(&self.render_segment_telemetry(pending_changes));
        out
    }

    /// #2519: renders `lumen_search_latency_seconds_bucket{le=...}`
    /// (cumulative, per the Prometheus histogram convention) followed by
    /// `_sum` and `_count`, all under one `# HELP`/`# TYPE histogram`
    /// declaration. See [`SEARCH_LATENCY_BUCKETS_US`] for the bucket
    /// bounds and `search_latency_ms_count`'s doc for why `_count`/`+Inf`
    /// reuse that field instead of a dedicated histogram-count atomic.
    fn render_search_latency_histogram(&self) -> String {
        const NAME: &str = "lumen_search_latency_seconds";
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# HELP {NAME} Search latency histogram in seconds, bucketed for search SLOs."
        );
        let _ = writeln!(out, "# TYPE {NAME} histogram");
        let mut cumulative = 0u64;
        for ((le, _bound_us), bucket) in SEARCH_LATENCY_BUCKETS_US
            .iter()
            .zip(self.search_latency_buckets.iter())
        {
            cumulative += bucket.get();
            let _ = writeln!(out, "{NAME}_bucket{{le=\"{le}\"}} {cumulative}");
        }
        let total = self.search_latency_ms_count.get();
        let _ = writeln!(out, "{NAME}_bucket{{le=\"+Inf\"}} {total}");
        let sum_seconds = self.search_latency_us_sum.get() as f64 / 1_000_000.0;
        let _ = writeln!(out, "{NAME}_sum {sum_seconds}");
        let _ = writeln!(out, "{NAME}_count {total}");
        out
    }

    /// Append durable segment telemetry after every established metric family.
    /// The integer atomics use microseconds internally; the published metric
    /// names promise seconds, so conversion happens once here at scrape time.
    fn render_segment_telemetry(&self, pending_changes: PendingChangeAccounting) -> String {
        let samples = [
            Sample::new("lumen_text_row_stage_rows_total", "counter",
                "Total immutable Text rows successfully prepared by this Engine.", self.text_row_stage_rows_total.get()),
            Sample::new("lumen_text_row_stage_input_bytes_total", "counter",
                "Total input bytes in immutable Text rows successfully prepared by this Engine.", self.text_row_stage_input_bytes_total.get()),
            Sample::new(
                "lumen_segment_checkpoint_completed_total",
                "counter",
                "Total successfully completed durable segment checkpoints.",
                self.segment_checkpoint_completed_total.get(),
            ),
            Sample::new(
                "lumen_segment_checkpoint_started_total",
                "counter",
                "Total durable segment checkpoint attempts that entered the shared checkpoint wrapper.",
                self.segment_checkpoint_started_total.get(),
            ),
            Sample::new(
                "lumen_segment_checkpoint_failed_total",
                "counter",
                "Total durable segment checkpoint attempts that returned an error from the shared checkpoint wrapper.",
                self.segment_checkpoint_failed_total.get(),
            ),
            Sample::new(
                "lumen_segment_checkpoint_in_flight",
                "gauge",
                "Current durable segment checkpoint attempts inside the shared checkpoint wrapper.",
                self.segment_checkpoint_in_flight.get(),
            ),
            Sample::new(
                "lumen_segment_checkpoint_bytes_total",
                "counter",
                "Total actual bytes durably published by completed segment checkpoints.",
                self.segment_checkpoint_bytes_total.get(),
            ),
            Sample::new(
                "lumen_segment_merge_completed_total",
                "counter",
                "Total successfully completed durable segment merges.",
                self.segment_merge_completed_total.get(),
            ),
            Sample::new(
                "lumen_segment_merge_read_bytes_total",
                "counter",
                "Total bytes read by successfully completed durable segment merges.",
                self.segment_merge_read_bytes_total.get(),
            ),
            Sample::new(
                "lumen_segment_merge_write_bytes_total",
                "counter",
                "Total bytes written by successfully completed durable segment merges.",
                self.segment_merge_write_bytes_total.get(),
            ),
            Sample::new(
                "lumen_segment_merge_linked_files_total",
                "counter",
                "Total files hard-linked by background segment merge jobs across both whole-root link passes.",
                self.segment_merge_linked_files_total.get(),
            ),
            Sample::new(
                "lumen_segment_merge_fields_total",
                "counter",
                "Total fields compacted by background segment merge jobs.",
                self.segment_merge_fields_total.get(),
            ),
            Sample::new(
                "lumen_segment_pending_delta_bytes",
                "gauge",
                "Current bytes in unmerged durable segment delta layers.",
                self.segment_pending_delta_bytes.get(),
            ),
            Sample::new(
                "lumen_segment_pending_delta_layers",
                "gauge",
                "Current count of unmerged durable segment delta layers.",
                self.segment_pending_delta_layers.get(),
            ),
            Sample::new(
                "lumen_segment_backpressure_total",
                "counter",
                "Total admissions delayed or refused by real segment backpressure.",
                self.segment_backpressure_total.get(),
            ),
            Sample::new(
                "lumen_segment_disk_bytes",
                "gauge",
                "Current durable segment files on disk in bytes.",
                self.segment_disk_bytes.get(),
            ),
            Sample::new(
                "lumen_process_rss_high_water_bytes",
                "gauge",
                "Linux process peak resident set size from /proc/self/status VmHWM in bytes; require lumen_process_rss_high_water_available=1.",
                self.process_rss_high_water_bytes.get(),
            ),
            Sample::new(
                "lumen_process_rss_high_water_available",
                "gauge",
                "1 when this scrape parsed Linux /proc/self/status VmHWM, 0 when unavailable.",
                self.process_rss_high_water_available.get(),
            ),
            Sample::new(
                "lumen_pending_change_reserved_bytes",
                "gauge",
                "Process-wide bytes reserved before apply for pending durable changes.",
                pending_changes.reserved,
            ),
            Sample::new(
                "lumen_pending_change_active_bytes",
                "gauge",
                "Process-wide active bytes owned by pending durable changes.",
                pending_changes.active,
            ),
            Sample::new(
                "lumen_pending_change_frozen_bytes",
                "gauge",
                "Process-wide frozen bytes retained by incomplete durable checkpoints.",
                pending_changes.frozen,
            ),
            Sample::new(
                "lumen_pending_change_total_bytes",
                "gauge",
                "Process-wide pending durable-change bytes across all ownership states.",
                pending_changes.total,
            ),
            Sample::new(
                "lumen_pending_change_high_water_bytes",
                "gauge",
                "Largest observed process-wide pending durable-change total in bytes.",
                pending_changes.high_water,
            ),
        ];
        let mut out = metrics_prometheus::render(&samples);
        render_duration_histogram(
            &mut out,
            "lumen_segment_checkpoint_duration_seconds",
            "Total duration of completed durable segment checkpoints in seconds.",
            self.segment_checkpoint_duration_us_sum.get(),
            self.segment_checkpoint_duration_count.get(),
        );
        render_duration_histogram(
            &mut out,
            "lumen_segment_capture_lock_seconds",
            "Total capture-lock hold time of completed durable segment checkpoints in seconds.",
            self.segment_capture_lock_us_sum.get(),
            self.segment_capture_lock_count.get(),
        );
        render_duration_histogram(
            &mut out,
            "lumen_segment_merge_save_gate_held_seconds",
            "Total time background segment merge jobs held the publication save gate in seconds.",
            self.segment_merge_save_gate_us_sum.get(),
            self.segment_merge_save_gate_count.get(),
        );
        out.push_str(&self.render_merge_phase_breakdown());
        out.push_str(&self.render_coordinator_apply_histogram());
        out.push_str(&self.render_coordinator_stage_histogram());
        render_apply_duration_histogram(
            &mut out,
            "lumen_engine_state_write_lock_wait_seconds",
            "Time spent waiting to acquire the committed-apply engine state write lock, in seconds.",
            &self.engine_state_write_lock_wait_seconds_buckets,
            &self.engine_state_write_lock_wait_seconds_us_sum,
            &self.engine_state_write_lock_wait_seconds_count,
        );
        render_apply_duration_histogram(
            &mut out,
            "lumen_engine_state_write_lock_held_seconds",
            "Time the committed-apply engine state write lock was held, in seconds.",
            &self.engine_state_write_lock_held_seconds_buckets,
            &self.engine_state_write_lock_held_seconds_us_sum,
            &self.engine_state_write_lock_held_seconds_count,
        );
        render_apply_duration_histogram(
            &mut out,
            "lumen_hnsw_add_seconds",
            "Time spent in live HNSW graph additions during committed apply, in seconds.",
            &self.hnsw_add_seconds_buckets,
            &self.hnsw_add_seconds_us_sum,
            &self.hnsw_add_seconds_count,
        );
        render_apply_duration_histogram(
            &mut out,
            "lumen_hnsw_graph_rebuild_seconds",
            "Time spent rebuilding an HNSW graph during a live add, in seconds.",
            &self.hnsw_graph_rebuild_seconds_buckets,
            &self.hnsw_graph_rebuild_seconds_us_sum,
            &self.hnsw_graph_rebuild_seconds_count,
        );
        render_apply_duration_histogram(
            &mut out,
            "lumen_hnsw_write_lock_wait_seconds",
            "Time spent waiting to acquire an HNSW graph write lock, in seconds.",
            &self.hnsw_write_lock_wait_seconds_buckets,
            &self.hnsw_write_lock_wait_seconds_us_sum,
            &self.hnsw_write_lock_wait_seconds_count,
        );
        render_apply_duration_histogram(
            &mut out,
            "lumen_hnsw_write_lock_held_seconds",
            "Time an HNSW graph write lock was held, in seconds.",
            &self.hnsw_write_lock_held_seconds_buckets,
            &self.hnsw_write_lock_held_seconds_us_sum,
            &self.hnsw_write_lock_held_seconds_count,
        );
        render_apply_duration_histogram(
            &mut out,
            "lumen_segment_capacity_relief_checkpoint_merge_seconds",
            "Time for successful capacity-relief checkpoint-plus-merge cycles, in seconds.",
            &self.segment_capacity_relief_checkpoint_merge_seconds_buckets,
            &self.segment_capacity_relief_checkpoint_merge_seconds_us_sum,
            &self.segment_capacity_relief_checkpoint_merge_seconds_count,
        );
        out
    }

    fn render_coordinator_stage_histogram(&self) -> String {
        const NAME: &str = "lumen_coordinator_stage_seconds";
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# HELP {NAME} Time spent in each admitted coordinator request stage, in seconds."
        );
        let _ = writeln!(out, "# TYPE {NAME} histogram");
        for stage in CoordinatorStage::ALL {
            for kind in ApplyKind::ALL {
                let stage_idx = stage.index();
                let kind_idx = kind.index();
                let mut cumulative = 0u64;
                for ((le, _), bucket) in APPLY_SECONDS_BUCKETS_US
                    .iter()
                    .zip(self.coordinator_stage_seconds_buckets[stage_idx][kind_idx].iter())
                {
                    cumulative += bucket.get();
                    let _ = writeln!(
                        out,
                        "{NAME}_bucket{{le=\"{le}\",kind=\"{}\",stage=\"{}\"}} {cumulative}",
                        kind.label(),
                        stage.label()
                    );
                }
                let total = self.coordinator_stage_seconds_count[stage_idx][kind_idx].get();
                let _ = writeln!(
                    out,
                    "{NAME}_bucket{{le=\"+Inf\",kind=\"{}\",stage=\"{}\"}} {total}",
                    kind.label(),
                    stage.label()
                );
                let sum = self.coordinator_stage_seconds_us_sum[stage_idx][kind_idx].get() as f64
                    / 1_000_000.0;
                let _ = writeln!(
                    out,
                    "{NAME}_sum{{kind=\"{}\",stage=\"{}\"}} {sum}",
                    kind.label(),
                    stage.label()
                );
                let _ = writeln!(
                    out,
                    "{NAME}_count{{kind=\"{}\",stage=\"{}\"}} {total}",
                    kind.label(),
                    stage.label()
                );
            }
        }
        out
    }

    /// #4326: render the `kind`-labelled `lumen_coordinator_apply_seconds`
    /// histogram and its `lumen_coordinator_apply_items_total` sibling
    /// counter — see [`Metrics::observe_coordinator_apply`].
    ///
    /// Hand-rolled for the same reason `render_search_latency_histogram`
    /// and `render_merge_phase_breakdown` are: a histogram's
    /// `_sum`/`_count`/`_bucket` names all suffix ONE base name under a
    /// single `# HELP`/`# TYPE histogram` pair, which
    /// `metrics_prometheus::render_labeled` does not model. Every kind in
    /// [`ApplyKind::ALL`] emits a row even at zero, so a scrape config
    /// never has to tolerate a kind appearing only after the first write of
    /// that shape.
    fn render_coordinator_apply_histogram(&self) -> String {
        const SECONDS: &str = "lumen_coordinator_apply_seconds";
        const ITEMS: &str = "lumen_coordinator_apply_items_total";
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# HELP {SECONDS} Time the write coordinator spent applying one admitted local record, in seconds."
        );
        let _ = writeln!(out, "# TYPE {SECONDS} histogram");
        for kind in ApplyKind::ALL {
            let idx = kind.index();
            let label = kind.label();
            let mut cumulative = 0u64;
            for ((le, _bound_us), bucket) in APPLY_SECONDS_BUCKETS_US
                .iter()
                .zip(self.coordinator_apply_seconds_buckets[idx].iter())
            {
                cumulative += bucket.get();
                let _ = writeln!(
                    out,
                    "{SECONDS}_bucket{{le=\"{le}\",kind=\"{label}\"}} {cumulative}"
                );
            }
            let total = self.coordinator_apply_seconds_count[idx].get();
            let _ = writeln!(
                out,
                "{SECONDS}_bucket{{le=\"+Inf\",kind=\"{label}\"}} {total}"
            );
            let sum_seconds = self.coordinator_apply_seconds_us_sum[idx].get() as f64 / 1_000_000.0;
            let _ = writeln!(out, "{SECONDS}_sum{{kind=\"{label}\"}} {sum_seconds}");
            let _ = writeln!(out, "{SECONDS}_count{{kind=\"{label}\"}} {total}");
        }
        let _ = writeln!(
            out,
            "# HELP {ITEMS} Items applied per write-coordinator apply, by RaftLogEntry kind."
        );
        let _ = writeln!(out, "# TYPE {ITEMS} counter");
        for kind in ApplyKind::ALL {
            let _ = writeln!(
                out,
                "{ITEMS}{{kind=\"{}\"}} {}",
                kind.label(),
                self.coordinator_apply_items_total[kind.index()].get()
            );
        }
        out
    }

    /// Render the per-[`MergeStep`] cost breakdown of background segment
    /// merge jobs: a `phase`-labelled histogram of step durations plus a
    /// `phase`-labelled counter of the filesystem entries each step touched.
    ///
    /// Hand-rolled for the same reason `render_search_latency_histogram` is:
    /// a histogram's `_sum`/`_count`/`_bucket` names all suffix ONE base name
    /// under a single `# HELP`/`# TYPE histogram` pair, which
    /// `metrics_prometheus::render_labeled` does not model. Every step in
    /// [`MergeStep::ALL`] emits a row even at zero, so a scrape config never
    /// has to tolerate a phase appearing only after the first slow job.
    fn render_merge_phase_breakdown(&self) -> String {
        const SECONDS: &str = "lumen_segment_merge_phase_seconds";
        const FILES: &str = "lumen_segment_merge_phase_files_total";
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# HELP {SECONDS} Per-phase duration of background segment merge jobs in seconds."
        );
        let _ = writeln!(out, "# TYPE {SECONDS} histogram");
        for step in MergeStep::ALL {
            let (micros, count, _) = self.segment_merge_step_observation(step);
            let phase = step.name();
            let seconds = micros as f64 / 1_000_000.0;
            let _ = writeln!(
                out,
                "{SECONDS}_bucket{{le=\"+Inf\",phase=\"{phase}\"}} {count}"
            );
            let _ = writeln!(out, "{SECONDS}_sum{{phase=\"{phase}\"}} {seconds}");
            let _ = writeln!(out, "{SECONDS}_count{{phase=\"{phase}\"}} {count}");
        }
        let _ = writeln!(
            out,
            "# HELP {FILES} Filesystem entries touched per phase by background segment merge jobs."
        );
        let _ = writeln!(out, "# TYPE {FILES} counter");
        for step in MergeStep::ALL {
            let (_, _, files) = self.segment_merge_step_observation(step);
            let _ = writeln!(out, "{FILES}{{phase=\"{}\"}} {files}", step.name());
        }
        out
    }
}

/// #2519: `LUMEN_SLOW_QUERY_MS` (milliseconds) if set and parseable to a
/// `u64`, else [`DEFAULT_SLOW_QUERY_THRESHOLD_MS`]. Read once at
/// `Metrics::new()` construction — mirrors `hnsw_search_ef`'s
/// read-once-per-construction convention in `src/vector_index.rs`.
fn slow_query_threshold_ms_from_env() -> u64 {
    std::env::var("LUMEN_SLOW_QUERY_MS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_SLOW_QUERY_THRESHOLD_MS)
}

/// #2475: current unix-epoch seconds, saturating to `0` on a pre-epoch
/// clock rather than panicking (metrics rendering must never fail).
fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn duration_to_micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

/// Record one observation in an unlabelled histogram that shares the bounded
/// apply-latency buckets. The stored buckets are exclusive; rendering makes
/// them cumulative as Prometheus requires.
fn observe_apply_duration_histogram(
    buckets: &[Counter; APPLY_SECONDS_BUCKET_COUNT],
    micros_sum: &Counter,
    count: &Counter,
    elapsed: Duration,
) {
    let micros = duration_to_micros(elapsed);
    if let Some(bucket_idx) = APPLY_SECONDS_BUCKETS_US
        .iter()
        .position(|&(_, bound_us)| micros <= bound_us)
    {
        buckets[bucket_idx].incr();
    }
    micros_sum.add(micros);
    count.incr();
}

/// Merge a stack-local batch of committed-apply observations after its state
/// write lock has dropped. Each bucket remains an exclusive bucket.
fn observe_apply_duration_observations(
    buckets: &[Counter; APPLY_SECONDS_BUCKET_COUNT],
    micros_sum: &Counter,
    count: &Counter,
    observations: &ApplyDurationObservations,
) {
    for (bucket, observed) in buckets.iter().zip(observations.buckets) {
        bucket.add(observed);
    }
    micros_sum.add(observations.micros_sum);
    count.add(observations.count);
}

/// Render one unlabelled bounded histogram that uses
/// [`APPLY_SECONDS_BUCKETS_US`] and an explicit `+Inf` count.
fn render_apply_duration_histogram(
    out: &mut String,
    name: &str,
    help: &str,
    buckets: &[Counter; APPLY_SECONDS_BUCKET_COUNT],
    micros_sum: &Counter,
    count: &Counter,
) {
    let _ = writeln!(out, "# HELP {name} {help}");
    let _ = writeln!(out, "# TYPE {name} histogram");
    let mut cumulative = 0u64;
    for ((le, _), bucket) in APPLY_SECONDS_BUCKETS_US.iter().zip(buckets.iter()) {
        cumulative += bucket.get();
        let _ = writeln!(out, "{name}_bucket{{le=\"{le}\"}} {cumulative}");
    }
    let total = count.get();
    let _ = writeln!(out, "{name}_bucket{{le=\"+Inf\"}} {total}");
    let sum_seconds = micros_sum.get() as f64 / 1_000_000.0;
    let _ = writeln!(out, "{name}_sum {sum_seconds}");
    let _ = writeln!(out, "{name}_count {total}");
}

fn render_duration_histogram(
    out: &mut String,
    base: &str,
    help: &str,
    micros_sum: u64,
    count: u64,
) {
    let seconds = micros_sum as f64 / 1_000_000.0;
    let _ = writeln!(out, "# HELP {base} {help}");
    let _ = writeln!(out, "# TYPE {base} histogram");
    let _ = writeln!(out, "{base}_bucket{{le=\"+Inf\"}} {count}");
    let _ = writeln!(out, "{base}_sum {seconds}");
    let _ = writeln!(out, "{base}_count {count}");
}

/// Parse the Linux `/proc/<pid>/status` `VmHWM` row. Linux reports this
/// value in exact `kB` units; any missing row, wrong unit, malformed value,
/// duplicate fields, or multiplication overflow is unavailable rather than a
/// fabricated byte count.
fn parse_linux_vmhwm_bytes(status: &str) -> Option<u64> {
    let mut saw_vmhwm = false;
    let mut bytes = None;
    for line in status.lines() {
        let Some(value) = line.strip_prefix("VmHWM:") else {
            continue;
        };
        if saw_vmhwm {
            return None;
        }
        saw_vmhwm = true;
        let mut words = value.split_ascii_whitespace();
        let kibibytes = words.next()?.parse::<u64>().ok()?;
        if words.next()? != "kB" || words.next().is_some() {
            return None;
        }
        bytes = kibibytes.checked_mul(1024);
    }
    bytes
}

fn process_rss_high_water_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| parse_linux_vmhwm_bytes(&status))
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinator_stage_histogram_renders_kind_and_stage_labels() {
        let metrics = Metrics::new();
        for stage in CoordinatorStage::ALL {
            metrics.observe_coordinator_stage(ApplyKind::Index, stage, Duration::from_millis(2));
        }
        let out = metrics.render();
        for stage in CoordinatorStage::ALL {
            let label = stage.label();
            assert!(out.contains(&format!(
                "lumen_coordinator_stage_seconds_count{{kind=\"index\",stage=\"{label}\"}} 1"
            )));
            assert!(out.contains(&format!(
                "lumen_coordinator_stage_seconds_sum{{kind=\"index\",stage=\"{label}\"}} 0.002"
            )));
        }
        assert!(out.contains("# TYPE lumen_coordinator_stage_seconds histogram"));
    }

    #[test]
    fn committed_apply_wait_and_hnsw_add_histograms_render_fixed_buckets() {
        let metrics = Metrics::new();
        let initial = metrics.render();
        for expected in [
            "lumen_engine_state_write_lock_wait_seconds_bucket{le=\"0.001\"} 0",
            "lumen_engine_state_write_lock_wait_seconds_bucket{le=\"+Inf\"} 0",
            "lumen_engine_state_write_lock_wait_seconds_sum 0",
            "lumen_engine_state_write_lock_wait_seconds_count 0",
            "lumen_hnsw_add_seconds_bucket{le=\"0.001\"} 0",
            "lumen_hnsw_add_seconds_bucket{le=\"+Inf\"} 0",
            "lumen_hnsw_add_seconds_sum 0",
            "lumen_hnsw_add_seconds_count 0",
        ] {
            assert!(
                initial.contains(expected),
                "missing initial {expected:?} in:\n{initial}"
            );
        }

        metrics.observe_engine_state_write_lock_wait(Duration::from_millis(2));
        metrics.observe_hnsw_add(Duration::from_secs(11));
        let out = metrics.render();
        for expected in [
            "lumen_engine_state_write_lock_wait_seconds_bucket{le=\"0.001\"} 0",
            "lumen_engine_state_write_lock_wait_seconds_bucket{le=\"0.005\"} 1",
            "lumen_engine_state_write_lock_wait_seconds_bucket{le=\"10\"} 1",
            "lumen_engine_state_write_lock_wait_seconds_bucket{le=\"+Inf\"} 1",
            "lumen_engine_state_write_lock_wait_seconds_sum 0.002",
            "lumen_engine_state_write_lock_wait_seconds_count 1",
            "lumen_hnsw_add_seconds_bucket{le=\"0.001\"} 0",
            "lumen_hnsw_add_seconds_bucket{le=\"10\"} 0",
            "lumen_hnsw_add_seconds_bucket{le=\"+Inf\"} 1",
            "lumen_hnsw_add_seconds_sum 11",
            "lumen_hnsw_add_seconds_count 1",
        ] {
            assert!(
                out.contains(expected),
                "missing observed {expected:?} in:\n{out}"
            );
        }
    }

    #[test]
    fn capacity_timing_histograms_split_lock_wait_hold_and_relief_cycle() {
        let metrics = Metrics::new();
        metrics.observe_engine_state_write_lock_hold(Duration::from_millis(3));
        metrics.observe_hnsw_write_lock_wait(Duration::from_millis(5));
        metrics.observe_hnsw_write_lock_hold(Duration::from_millis(7));
        metrics.observe_hnsw_graph_rebuild(Duration::from_millis(9));
        metrics.observe_capacity_relief_checkpoint_merge_cycle(Duration::from_millis(11));

        let out = metrics.render();
        for expected in [
            "lumen_engine_state_write_lock_held_seconds_sum 0.003",
            "lumen_engine_state_write_lock_held_seconds_count 1",
            "lumen_hnsw_write_lock_wait_seconds_sum 0.005",
            "lumen_hnsw_write_lock_wait_seconds_count 1",
            "lumen_hnsw_write_lock_held_seconds_sum 0.007",
            "lumen_hnsw_write_lock_held_seconds_count 1",
            "lumen_hnsw_graph_rebuild_seconds_sum 0.009",
            "lumen_hnsw_graph_rebuild_seconds_count 1",
            "lumen_segment_capacity_relief_checkpoint_merge_seconds_sum 0.011",
            "lumen_segment_capacity_relief_checkpoint_merge_seconds_count 1",
        ] {
            assert!(out.contains(expected), "missing {expected:?} in:\n{out}");
        }
    }

    #[test]
    fn checkpoint_attempt_metrics_render_initial_and_failed_values() {
        let metrics = Metrics::new();
        let initial = metrics.render();
        for expected in [
            "lumen_segment_checkpoint_started_total 0",
            "lumen_segment_checkpoint_failed_total 0",
            "lumen_segment_checkpoint_in_flight 0",
        ] {
            assert!(
                initial.contains(expected),
                "missing initial {expected:?} in:\n{initial}"
            );
        }

        metrics.start_segment_checkpoint_attempt();
        assert_eq!(metrics.segment_checkpoint_in_flight.get(), 1);
        metrics.finish_segment_checkpoint_attempt(true);
        let out = metrics.render();
        for expected in [
            "lumen_segment_checkpoint_started_total 1",
            "lumen_segment_checkpoint_failed_total 1",
            "lumen_segment_checkpoint_in_flight 0",
        ] {
            assert!(
                out.contains(expected),
                "missing observed {expected:?} in:\n{out}"
            );
        }
    }

    #[test]
    fn render_emits_every_metric() {
        let m = Metrics::new();
        m.incr_index(3, 100);
        m.observe_search(Duration::from_millis(7));
        m.set_raft_leader_known(2, true);
        m.set_reshard_fence_active(true);
        let out = m.render();
        for name in [
            "lumen_index_writes_total",
            // #2519: deprecated back-compat series must still be present.
            "lumen_search_latency_ms_sum",
            "lumen_search_latency_ms_count",
            // #2519: the new histogram + slow-query series.
            "lumen_search_latency_seconds_bucket",
            "lumen_search_latency_seconds_sum",
            "lumen_search_latency_seconds_count",
            "lumen_slow_queries_total",
            "lumen_storage_bytes",
            "lumen_posting_cache_hits_total",
            "lumen_replace_fields_skipped_total",
            "lumen_shard_map_version",
            "lumen_scatter_map_version_mismatches_total",
            "lumen_reshard_fence_active",
            "lumen_reshard_fence_armed_unixtime",
            "lumen_raft_leader_known",
            "lumen_storage_degraded",
            "lumen_storage_full_errors_total",
            "lumen_segment_checkpoint_completed_total",
            "lumen_segment_checkpoint_started_total",
            "lumen_segment_checkpoint_failed_total",
            "lumen_segment_checkpoint_in_flight",
            "lumen_segment_checkpoint_duration_seconds_count",
            "lumen_segment_checkpoint_duration_seconds_sum",
            "lumen_segment_capture_lock_seconds_count",
            "lumen_segment_capture_lock_seconds_sum",
            "lumen_segment_checkpoint_bytes_total",
            "lumen_segment_merge_completed_total",
            "lumen_segment_merge_read_bytes_total",
            "lumen_segment_merge_write_bytes_total",
            "lumen_segment_pending_delta_bytes",
            "lumen_segment_pending_delta_layers",
            "lumen_segment_backpressure_total",
            "lumen_segment_disk_bytes",
            "lumen_process_rss_high_water_bytes",
            "lumen_process_rss_high_water_available",
            "lumen_pending_change_reserved_bytes",
            "lumen_pending_change_active_bytes",
            "lumen_pending_change_frozen_bytes",
            "lumen_pending_change_total_bytes",
            "lumen_pending_change_high_water_bytes",
        ] {
            assert!(out.contains(name), "expected {name} in:\n{out}");
        }
        assert!(
            out.contains("lumen_raft_leader_known{shard=\"2\"} 1"),
            "expected labeled raft series in:\n{out}"
        );
    }

    fn rendered_pending_gauge(rendered: &str, name: &str) -> u64 {
        rendered
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{name} ")))
            .unwrap_or_else(|| panic!("missing {name} in:\n{rendered}"))
            .parse()
            .unwrap_or_else(|_| panic!("non-integer {name} in:\n{rendered}"))
    }

    fn assert_coherent_pending_change_gauges(rendered: &str, held_bytes: u64) {
        let reserved = rendered_pending_gauge(rendered, "lumen_pending_change_reserved_bytes");
        let active = rendered_pending_gauge(rendered, "lumen_pending_change_active_bytes");
        let frozen = rendered_pending_gauge(rendered, "lumen_pending_change_frozen_bytes");
        let total = rendered_pending_gauge(rendered, "lumen_pending_change_total_bytes");
        let high_water = rendered_pending_gauge(rendered, "lumen_pending_change_high_water_bytes");
        assert_eq!(
            reserved + active + frozen,
            total,
            "mixed scrape:\n{rendered}"
        );
        assert!(high_water >= total, "peak below live total:\n{rendered}");
        assert!(
            total >= held_bytes,
            "the held test reservation is absent from the scrape:\n{rendered}"
        );
    }

    #[test]
    fn pending_change_render_stays_coherent_during_concurrent_writes_and_scrapes() {
        let budget = ChangeBudget::process_shared();
        let held_owner = budget.owner();
        let held = held_owner.try_reserve(7).unwrap();
        let metrics = std::sync::Arc::new(Metrics::new());
        let gate = std::sync::Arc::new(std::sync::Barrier::new(3));
        let successes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        std::thread::scope(|scope| {
            let writer_budget = budget.clone();
            let writer_gate = gate.clone();
            let writer_successes = successes.clone();
            scope.spawn(move || {
                let owner = writer_budget.owner();
                writer_gate.wait();
                for _ in 0..256 {
                    if let Ok(reservation) = owner.try_reserve(1) {
                        writer_successes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        drop(reservation);
                    }
                }
            });
            for _ in 0..2 {
                let metrics = metrics.clone();
                let reader_gate = gate.clone();
                scope.spawn(move || {
                    reader_gate.wait();
                    for _ in 0..256 {
                        assert_coherent_pending_change_gauges(&metrics.render(), 7);
                    }
                });
            }
        });

        assert!(
            successes.load(std::sync::atomic::Ordering::Relaxed) > 0,
            "the writer must make real process-shared reservations"
        );
        drop(held);
    }

    #[test]
    fn pending_change_accounting_helper_maps_one_isolated_budget_state() {
        let budget = ChangeBudget::with_hard_limit(64);
        let owner = budget.owner();
        let active = owner.try_reserve(11).unwrap().commit().unwrap();
        let frozen = owner.freeze().unwrap();
        let reserved = owner.try_reserve(7).unwrap();
        let metrics = Metrics::new();

        let values = metrics.read_pending_change_accounting(&budget);
        assert_eq!(values.reserved, 7);
        assert_eq!(values.active, 0);
        assert_eq!(values.frozen, 11);
        assert_eq!(values.total, 18);
        assert_eq!(values.high_water, 18);

        drop(reserved);
        drop(active);
        assert_eq!(frozen.publish().unwrap(), 11);
    }

    /// #2519: a search observation lands in the correct cumulative
    /// histogram bucket, and every bucket >= its own falls in too (a
    /// Prometheus histogram bucket is "le", not exclusive).
    #[test]
    fn observe_search_updates_histogram_buckets() {
        let m = Metrics::new();
        m.observe_search(Duration::from_micros(1_500)); // 1.5ms -> le=0.0025 bucket
        let out = m.render();
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"0.001\"} 0"),
            "1.5ms observation must not count in the 1ms bucket:\n{out}"
        );
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"0.0025\"} 1"),
            "1.5ms observation must count in the 2.5ms bucket:\n{out}"
        );
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"5\"} 1"),
            "cumulative buckets past the observation's own bucket must include it:\n{out}"
        );
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"+Inf\"} 1"),
            "+Inf bucket must equal the total observation count:\n{out}"
        );
        assert!(
            out.contains("lumen_search_latency_seconds_count 1"),
            "histogram _count must equal the total observation count:\n{out}"
        );
    }

    /// #2519 AC: an artificially-slow observation (here, `Duration::MAX`
    /// against a default threshold) increments `lumen_slow_queries_total`,
    /// and it stays folded into the `+Inf` bucket rather than a stored
    /// 13th bucket counter.
    #[test]
    fn observe_search_increments_slow_queries_over_threshold() {
        let m = Metrics::new();
        m.observe_search(Duration::from_secs(10)); // past every finite bucket
        assert_eq!(m.slow_queries_total.get(), 1);
        let out = m.render();
        assert!(out.contains("lumen_slow_queries_total 1"), "{out}");
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"5\"} 0"),
            "a 10s observation must not land in the 5s bucket:\n{out}"
        );
        assert!(out.contains("lumen_search_latency_seconds_bucket{le=\"+Inf\"} 1"));
    }

    // Process-global env mutex shared across LUMEN_SLOW_QUERY_MS-mutating
    // tests (mirrors `auth.rs`'s `AUTH_ENV_LOCK`).
    static SLOW_QUERY_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// #2519 AC: the "threshold=0 trick" — `LUMEN_SLOW_QUERY_MS=0` makes
    /// every search, however fast, count as slow. `Metrics::new()` reads
    /// the env var once at construction, so it must be set before that
    /// call.
    #[test]
    fn slow_query_threshold_zero_counts_every_search_as_slow() {
        let _g = SLOW_QUERY_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("LUMEN_SLOW_QUERY_MS", "0");
        }
        let m = Metrics::new();
        unsafe {
            std::env::remove_var("LUMEN_SLOW_QUERY_MS");
        }
        m.observe_search(Duration::from_micros(1));
        assert_eq!(
            m.slow_queries_total.get(),
            1,
            "a threshold=0 override must count even a ~0ms search as slow"
        );
    }

    /// A fast search under the default 500ms threshold must NOT count as
    /// slow (regression guard against an inverted comparison).
    #[test]
    fn fast_search_under_default_threshold_is_not_slow() {
        let _g = SLOW_QUERY_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("LUMEN_SLOW_QUERY_MS");
        }
        let m = Metrics::new();
        m.observe_search(Duration::from_millis(5));
        assert_eq!(m.slow_queries_total.get(), 0);
    }

    /// #2475: a pod whose raft election-state poller never ticked (every
    /// standalone/non-raft deployment) must not publish
    /// `lumen_raft_leader_known` at all — a permanent `0` there would be
    /// indistinguishable from a genuinely stuck leaderless shard to
    /// `LumenRaftLeaderAbsent`.
    #[test]
    fn render_omits_raft_leader_known_when_never_set() {
        let m = Metrics::new();
        let out = m.render();
        assert!(
            !out.contains("lumen_raft_leader_known"),
            "unexpected raft series in:\n{out}"
        );
    }

    /// #2516: `mark_storage_degraded` flips the gauge to `1` and counts the
    /// hit; `clear_storage_degraded` (the periodic re-probe) flips it back
    /// without touching the counter — the counter is a lifetime total, not
    /// a "currently degraded" signal.
    #[test]
    fn storage_degraded_marks_and_clears() {
        let m = Metrics::new();
        assert!(!m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 0);

        m.mark_storage_degraded();
        assert!(m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 1);

        // A second hit while already degraded still counts, but the gauge
        // stays at 1 (sticky, not a counter).
        m.mark_storage_degraded();
        assert!(m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 2);

        m.clear_storage_degraded();
        assert!(!m.is_storage_degraded());
        assert_eq!(
            m.storage_full_errors_total.get(),
            2,
            "clearing degraded mode must not reset the lifetime error counter"
        );
    }

    /// Byte-identical golden-render check (#974): fixed inputs must
    /// reproduce the exact pre-refactor `render()` capture, byte for
    /// byte — this is the AC2 contract the observability EC claim
    /// (`lumen_claim_observability_prometheus_metrics`) depends on.
    #[test]
    fn render_is_byte_identical_to_pre_refactor_capture() {
        let m = Metrics::new();
        m.incr_index(3, 100);
        m.observe_search(Duration::from_millis(7));
        m.observe_search(Duration::from_millis(9));
        m.incr_duplicates();
        m.incr_collection_created(4);
        m.set_storage_bytes(2048);
        m.posting_cache_hits_total.add(5);
        m.posting_cache_misses_total.add(2);
        m.incr_replace_skipped(6);
        m.set_shard_map_version(3);
        m.incr_scatter_map_version_mismatch();
        // #2475: reach past the wall-clock-stamping setters and set the raw
        // gauges directly so this golden capture stays deterministic.
        m.reshard_fence_active.set(1);
        m.reshard_fence_armed_unixtime.set(1_700_000_000);
        m.set_raft_leader_known(2, true);
        m.mark_storage_degraded();
        let out = m.render();
        let golden = "# HELP lumen_index_writes_total Total index items applied.\n\
# TYPE lumen_index_writes_total counter\n\
lumen_index_writes_total 3\n\
# HELP lumen_index_bytes_total Total bytes written across all field indexes.\n\
# TYPE lumen_index_bytes_total counter\n\
lumen_index_bytes_total 100\n\
# HELP lumen_search_requests_total Total search requests served.\n\
# TYPE lumen_search_requests_total counter\n\
lumen_search_requests_total 2\n\
# HELP lumen_search_latency_ms_sum DEPRECATED (#2519): sum of search latencies in milliseconds; kept for dashboard back-compat, see lumen_search_latency_seconds for the real histogram.\n\
# TYPE lumen_search_latency_ms_sum counter\n\
lumen_search_latency_ms_sum 16\n\
# HELP lumen_search_latency_ms_count DEPRECATED (#2519): count of search latency observations; kept for dashboard back-compat, see lumen_search_latency_seconds for the real histogram.\n\
# TYPE lumen_search_latency_ms_count counter\n\
lumen_search_latency_ms_count 2\n\
# HELP lumen_slow_queries_total Total search queries at/above the slow-query threshold (LUMEN_SLOW_QUERY_MS, default 500ms).\n\
# TYPE lumen_slow_queries_total counter\n\
lumen_slow_queries_total 0\n\
# HELP lumen_duplicates_requests_total Total duplicate-detection requests.\n\
# TYPE lumen_duplicates_requests_total counter\n\
lumen_duplicates_requests_total 1\n\
# HELP lumen_collections_created_total Total collections created or extended.\n\
# TYPE lumen_collections_created_total counter\n\
lumen_collections_created_total 1\n\
# HELP lumen_schema_fields_total Total field declarations registered.\n\
# TYPE lumen_schema_fields_total counter\n\
lumen_schema_fields_total 4\n\
# HELP lumen_storage_bytes Approximate bytes held by all in-memory field indexes.\n\
# TYPE lumen_storage_bytes gauge\n\
lumen_storage_bytes 2048\n\
# HELP lumen_posting_cache_hits_total Posting cache hit count (0 until LSM cache is wired).\n\
# TYPE lumen_posting_cache_hits_total counter\n\
lumen_posting_cache_hits_total 5\n\
# HELP lumen_posting_cache_misses_total Posting cache miss count.\n\
# TYPE lumen_posting_cache_misses_total counter\n\
lumen_posting_cache_misses_total 2\n\
# HELP lumen_replace_fields_skipped_total Total docs:replace fields skipped as unchanged no-ops.\n\
# TYPE lumen_replace_fields_skipped_total counter\n\
lumen_replace_fields_skipped_total 6\n\
# HELP lumen_shard_map_version This pod's live routed shard-map version (0 outside routed deployments).\n\
# TYPE lumen_shard_map_version gauge\n\
lumen_shard_map_version 3\n\
# HELP lumen_scatter_map_version_mismatches_total Scatter search sub-responses whose responding pod's map version differed from the sender's.\n\
# TYPE lumen_scatter_map_version_mismatches_total counter\n\
lumen_scatter_map_version_mismatches_total 1\n\
# HELP lumen_reshard_fence_active 1 while this pod believes a reshard-driver write fence is currently armed on it.\n\
# TYPE lumen_reshard_fence_active gauge\n\
lumen_reshard_fence_active 1\n\
# HELP lumen_reshard_fence_armed_unixtime Unix time the currently (or most recently) armed reshard write fence was armed at.\n\
# TYPE lumen_reshard_fence_armed_unixtime gauge\n\
lumen_reshard_fence_armed_unixtime 1700000000\n\
# HELP lumen_storage_degraded 1 while this node is in ENOSPC degraded read-only mode (a durable write path hit disk-full).\n\
# TYPE lumen_storage_degraded gauge\n\
lumen_storage_degraded 1\n\
# HELP lumen_storage_full_errors_total Total genuine ENOSPC hits observed on a durable write path.\n\
# TYPE lumen_storage_full_errors_total counter\n\
lumen_storage_full_errors_total 1\n\
# HELP lumen_search_latency_seconds Search latency histogram in seconds, bucketed for search SLOs.\n\
# TYPE lumen_search_latency_seconds histogram\n\
lumen_search_latency_seconds_bucket{le=\"0.001\"} 0\n\
lumen_search_latency_seconds_bucket{le=\"0.0025\"} 0\n\
lumen_search_latency_seconds_bucket{le=\"0.005\"} 0\n\
lumen_search_latency_seconds_bucket{le=\"0.01\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.025\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.05\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.1\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.25\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.5\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"1\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"2.5\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"5\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"+Inf\"} 2\n\
lumen_search_latency_seconds_sum 0.016\n\
lumen_search_latency_seconds_count 2\n\
# HELP lumen_raft_leader_known 1 while this pod's raft election-state poll believes its shard currently has an elected leader, 0 otherwise. Omitted for standalone/non-raft deployments.\n\
# TYPE lumen_raft_leader_known gauge\n\
lumen_raft_leader_known{shard=\"2\"} 1\n";
        assert!(
            out.starts_with(golden),
            "the established metric surface diverged before the appended telemetry (#2475 added \
             lumen_reshard_fence_active + lumen_reshard_fence_armed_unixtime + \
             lumen_raft_leader_known; \
             #2519 added lumen_slow_queries_total + the \
             lumen_search_latency_seconds histogram, and marked \
             lumen_search_latency_ms_sum/_count deprecated in HELP text; \
             #2516 added lumen_storage_degraded + lumen_storage_full_errors_total)"
        );
        let appended = &out[golden.len()..];
        for name in [
            "lumen_segment_checkpoint_completed_total",
            "lumen_segment_checkpoint_duration_seconds",
            "lumen_segment_capture_lock_seconds",
            "lumen_segment_checkpoint_bytes_total",
            "lumen_segment_merge_completed_total",
            "lumen_segment_merge_read_bytes_total",
            "lumen_segment_merge_write_bytes_total",
            "lumen_segment_pending_delta_bytes",
            "lumen_segment_pending_delta_layers",
            "lumen_segment_backpressure_total",
            "lumen_segment_disk_bytes",
            "lumen_process_rss_high_water_bytes",
            "lumen_process_rss_high_water_available",
            "lumen_pending_change_reserved_bytes",
            "lumen_pending_change_active_bytes",
            "lumen_pending_change_frozen_bytes",
            "lumen_pending_change_total_bytes",
            "lumen_pending_change_high_water_bytes",
            "lumen_coordinator_apply_seconds",
            "lumen_coordinator_apply_items_total",
            "lumen_engine_state_write_lock_wait_seconds",
            "lumen_hnsw_add_seconds",
        ] {
            assert!(
                appended.contains(name),
                "missing appended {name} in:\n{appended}"
            );
        }
    }

    /// #4326: `ApplyKind::from_entry`/`kind_label`/`apply_item_count` must
    /// agree with the `RaftLogEntry` variant they classify, and
    /// `observe_coordinator_apply` must land in the right `kind`'s
    /// per-kind atomic slot without disturbing a sibling kind.
    #[test]
    fn apply_kind_classifies_entries_and_counts_items() {
        use crate::log_entry::RaftLogEntry;
        use crate::types::{
            BatchUnindexDocsRequest, FieldValue, IndexItem, IndexRequest, ReplaceDocItem,
            ReplaceDocsRequest,
        };

        let index_entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: "1".into(),
                        field: "f".into(),
                        value: FieldValue::String("a".into()),
                        version: None,
                    },
                    IndexItem {
                        external_id: "2".into(),
                        field: "f".into(),
                        value: FieldValue::String("b".into()),
                        version: None,
                    },
                    IndexItem {
                        external_id: "3".into(),
                        field: "f".into(),
                        value: FieldValue::String("c".into()),
                        version: None,
                    },
                ],
                request_id: None,
            },
        };
        assert_eq!(kind_label(&index_entry), "index");
        assert_eq!(apply_item_count(&index_entry), 3);

        let replace_entry = RaftLogEntry::ReplaceDocs {
            collection_id: "c".into(),
            req: ReplaceDocsRequest {
                docs: vec![ReplaceDocItem {
                    external_id: "1".into(),
                    version: None,
                    fields: Default::default(),
                }],
            },
        };
        assert_eq!(kind_label(&replace_entry), "replace");
        assert_eq!(apply_item_count(&replace_entry), 1);

        let unindex_entry = RaftLogEntry::UnindexDocs {
            collection_id: "c".into(),
            req: BatchUnindexDocsRequest {
                external_ids: vec!["1".into(), "2".into()],
            },
        };
        assert_eq!(kind_label(&unindex_entry), "unindex");
        assert_eq!(apply_item_count(&unindex_entry), 2);

        let drop_field_entry = RaftLogEntry::DropField {
            collection_id: "c".into(),
            field_name: "f".into(),
        };
        assert_eq!(kind_label(&drop_field_entry), "drop_field");
        assert_eq!(apply_item_count(&drop_field_entry), 1);

        let m = Metrics::new();
        m.observe_coordinator_apply(ApplyKind::Index, 3, Duration::from_millis(2));
        m.observe_coordinator_apply(ApplyKind::UnindexDocs, 2, Duration::from_micros(500));

        let out = m.render();
        assert!(
            out.contains("lumen_coordinator_apply_seconds_count{kind=\"index\"} 1"),
            "missing index apply count in:\n{out}"
        );
        assert!(
            out.contains("lumen_coordinator_apply_items_total{kind=\"index\"} 3"),
            "missing index item total in:\n{out}"
        );
        assert!(
            out.contains("lumen_coordinator_apply_seconds_count{kind=\"unindex\"} 1"),
            "missing unindex apply count in:\n{out}"
        );
        assert!(
            out.contains("lumen_coordinator_apply_items_total{kind=\"unindex\"} 2"),
            "missing unindex item total in:\n{out}"
        );
        // Every other kind still emits its row, at zero, so a scrape config
        // never has to tolerate a kind appearing only after its first write.
        assert!(
            out.contains("lumen_coordinator_apply_seconds_count{kind=\"replace\"} 0"),
            "unobserved kind must still emit its zero row in:\n{out}"
        );
        assert!(
            out.contains("lumen_coordinator_apply_items_total{kind=\"replace\"} 0"),
            "unobserved kind must still emit its zero row in:\n{out}"
        );
    }

    /// A merge job's per-step cost must be readable from the same `/metrics`
    /// surface production scrapes: one labelled `phase` row per
    /// [`MergeStep`], plus the file and field counts that say whether a slow
    /// job was wide (root file count) or deep (payload).
    #[test]
    fn merge_step_observations_render_one_labelled_phase_row_per_step() {
        let m = Metrics::new();
        m.observe_segment_merge_step(MergeStep::CaptureBefore, Duration::from_micros(1_000), 0);
        m.observe_segment_merge_step(MergeStep::LinkScratch, Duration::from_micros(2_000), 7);
        m.observe_segment_merge_step(MergeStep::LinkGeneration, Duration::from_micros(4_000), 9);
        m.observe_segment_merge_step(MergeStep::Total, Duration::from_micros(8_000), 0);
        m.observe_segment_merge_save_gate(Duration::from_micros(6_000));
        m.incr_segment_merge_fields(3);

        assert_eq!(
            m.segment_merge_step_observation(MergeStep::LinkScratch),
            (2_000, 1, 7),
            "the registry must read back exactly what was observed"
        );
        assert_eq!(
            m.segment_merge_step_observation(MergeStep::Compact),
            (0, 0, 0),
            "an unobserved step reads back as zero, not as another step's value"
        );

        let out = m.render();
        for step in MergeStep::ALL {
            let name = step.name();
            assert!(
                out.contains(&format!(
                    "lumen_segment_merge_phase_seconds_count{{phase=\"{name}\"}}"
                )),
                "missing phase row {name} in:\n{out}"
            );
            assert!(
                out.contains(&format!(
                    "lumen_segment_merge_phase_files_total{{phase=\"{name}\"}}"
                )),
                "missing phase file row {name} in:\n{out}"
            );
        }
        for expected in [
            "lumen_segment_merge_phase_seconds_sum{phase=\"link_scratch\"} 0.002",
            "lumen_segment_merge_phase_seconds_count{phase=\"link_scratch\"} 1",
            "lumen_segment_merge_phase_files_total{phase=\"link_scratch\"} 7",
            "lumen_segment_merge_phase_seconds_sum{phase=\"total\"} 0.008",
            "lumen_segment_merge_phase_files_total{phase=\"compact\"} 0",
            "lumen_segment_merge_linked_files_total 16",
            "lumen_segment_merge_fields_total 3",
            "lumen_segment_merge_save_gate_held_seconds_sum 0.006",
            "lumen_segment_merge_save_gate_held_seconds_count 1",
        ] {
            assert!(out.contains(expected), "missing {expected:?} in:\n{out}");
        }
    }

    #[test]
    fn durable_segment_observations_render_required_totals_and_gauges() {
        let m = Metrics::new();
        m.observe_segment_checkpoint(13, Duration::from_micros(2_500), Duration::from_micros(500));
        m.observe_segment_checkpoint(19, Duration::from_micros(1_250), Duration::from_micros(750));
        m.observe_segment_merge(23, 29);
        m.observe_segment_merge(31, 37);
        m.set_segment_pending_delta(41, 2);
        m.incr_segment_backpressure();
        m.set_segment_disk_bytes(43);

        let out = m.render();
        for expected in [
            "lumen_segment_checkpoint_completed_total 2",
            "lumen_segment_checkpoint_bytes_total 32",
            "lumen_segment_checkpoint_duration_seconds_sum 0.00375",
            "lumen_segment_checkpoint_duration_seconds_count 2",
            "lumen_segment_capture_lock_seconds_sum 0.00125",
            "lumen_segment_capture_lock_seconds_count 2",
            "lumen_segment_merge_completed_total 2",
            "lumen_segment_merge_read_bytes_total 54",
            "lumen_segment_merge_write_bytes_total 66",
            "lumen_segment_pending_delta_bytes 41",
            "lumen_segment_pending_delta_layers 2",
            "lumen_segment_backpressure_total 1",
            "lumen_segment_disk_bytes 43",
        ] {
            assert!(out.contains(expected), "missing {expected:?} in:\n{out}");
        }
    }

    #[test]
    fn parse_linux_vmhwm_requires_one_exact_kilobyte_row() {
        assert_eq!(
            parse_linux_vmhwm_bytes("Name:\tlumen\nVmHWM:\t 123 kB\n"),
            Some(123 * 1024)
        );
        assert_eq!(parse_linux_vmhwm_bytes("Name:\tlumen\n"), None);
        assert_eq!(parse_linux_vmhwm_bytes("VmHWM:\tnope kB\n"), None);
        assert_eq!(parse_linux_vmhwm_bytes("VmHWM:\t123 KB\n"), None);
        assert_eq!(parse_linux_vmhwm_bytes("VmHWM:\t123 kB extra\n"), None);
        assert_eq!(
            parse_linux_vmhwm_bytes("VmHWM:\t18446744073709551615 kB\n"),
            None
        );
        assert_eq!(
            parse_linux_vmhwm_bytes("VmHWM:\t1 kB\nVmHWM:\t2 kB\n"),
            None
        );
        assert_eq!(
            parse_linux_vmhwm_bytes("VmHWM:\t18446744073709551615 kB\nVmHWM:\t2 kB\n"),
            None
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_render_reads_this_process_vmhwm_and_marks_it_available() {
        let expected = std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| parse_linux_vmhwm_bytes(&status))
            .expect("Linux /proc/self/status must expose a valid VmHWM row");
        let m = Metrics::new();
        let out = m.render();
        assert!(
            out.contains("lumen_process_rss_high_water_available 1"),
            "VmHWM was available but render did not mark it available:\n{out}"
        );
        let rendered = m.process_rss_high_water_bytes.get();
        assert!(
            rendered >= expected,
            "rendered VmHWM {rendered} fell below pre-render /proc value {expected}"
        );
        assert!(
            rendered > 0,
            "a running test process must have nonzero VmHWM"
        );
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn non_linux_render_marks_rss_high_water_unavailable() {
        let m = Metrics::new();
        let out = m.render();
        assert!(
            out.contains("lumen_process_rss_high_water_available 0"),
            "non-Linux must not qualify an unavailable VmHWM value:\n{out}"
        );
        assert!(out.contains("lumen_process_rss_high_water_bytes 0"));
    }
}
// CODEGEN-END
