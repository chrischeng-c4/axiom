---
id: projects-lumen-src-api-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    gap: "query-planner-boolean-eval-roaring-postings"
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
  - id: "long-running-stability"
    role: primary
    gap: "meta-api-health-ready-metrics-version"
    claim: "meta-api-health-ready-metrics-version"
    coverage: full
    rationale: "api.rs owns /healthz, /readyz, /metrics, and /version, which are the operability meta endpoints used by probes and scrapes."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/api.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/api.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ApiDoc` | projects/lumen/src/api.rs | struct | pub | 614 |  |
| `ApiErr` | projects/lumen/src/api.rs | struct | pub | 2368 |  |
| `AppState` | projects/lumen/src/api.rs | struct | pub | 64 |  |
| `RoutedBackend` | projects/lumen/src/api.rs | trait | pub | 278 | Cross-pod shard routing for operator/k8s serving pods (#1398 R1-R3); `AppState::routed` is `None` for every non-routed deployment. |
| `ShardForwardMisrouted` | projects/lumen/src/api.rs | struct | pub | 2459 | A forwarded request's one-hop marker claimed this pod, but recomputing ownership disagrees (#1442 R1) — a spoofed or genuinely misrouted forward, rejected rather than honored. |
| `ShardForwardRemoteError` | projects/lumen/src/api.rs | struct | pub | 2407 | The owning shard was reached and answered, but with a non-2xx status. |
| `ShardForwardUnavailable` | projects/lumen/src/api.rs | struct | pub | 2389 | One-hop shard-forward failure — the owning shard was unreachable or its response could not be decoded. |
| `ShardMapVersionMismatch` | projects/lumen/src/api.rs | struct | pub | 2431 | A forwarded request declared a shard-map version that disagrees with this pod's own live map (#1442 R2) — the rolling-restart mixed-map window. |
| `WriteFence` | projects/lumen/src/api.rs | struct | pub | 173 | #1396 R2: bounded write pause on still-moving virtual buckets during a reshard's final `CatchingUp` pass. Armed/cleared via `POST /admin/reshard:fence`; self-expires on its TTL deadline so a crashed driver can never leave a permanent fence. #1443 R3: `arm` now returns `bool` (false on `Instant + Duration` overflow instead of panicking) and the internal mutex is poison-proof (`unwrap_or_else(poisoned -> into_inner)`), so an overflowed/panicked prior caller can never wedge the fence permanently. |
| `new` | projects/lumen/src/api.rs | function | pub | 461 | new(engine: Arc<Engine>, auth: Arc<AuthConfig>) -> Self |
| `open` | projects/lumen/src/api.rs | function | pub | 499 | open(engine: Arc<Engine>) -> Self |
| `openapi` | projects/lumen/src/api.rs | function | pub | 2296 | openapi() -> utoipa::openapi::OpenApi |
| `router` | projects/lumen/src/api.rs | function | pub | 653 | router(state: AppState) -> Router |
| `with_checkpoint` | projects/lumen/src/api.rs | function | pub | 483 | with_checkpoint(mut self, checkpoint: Arc<dyn CheckpointSink>) -> Self |
| `with_cluster` | projects/lumen/src/api.rs | function | pub | 465 | with_cluster(mut self, cluster: Arc<crate::raft::ClusterState>) -> Self |
| `with_components` | projects/lumen/src/api.rs | function | pub | 437 | with_components(         engine: Arc<Engine>,         auth: Arc<AuthConfig>,         writer: Arc<dyn WriteSink>,     ) -> Self |
| `with_routed` | projects/lumen/src/api.rs | function | pub | 492 | with_routed(mut self, routed: Arc<dyn RoutedBackend>) -> Self |
| `with_search_backend` | projects/lumen/src/api.rs | function | pub | 470 | with_search_backend(mut self, search_backend: Arc<dyn SearchBackend>) -> Self |
| `with_wal` | projects/lumen/src/api.rs | function | pub | 429 | with_wal(engine: Arc<Engine>, auth: Arc<AuthConfig>, wal: SharedWal) -> Self |
| `with_write_backend` | projects/lumen/src/api.rs | function | pub | 475 | with_write_backend(mut self, write_backend: Arc<dyn WriteBackend>) -> Self |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! HTTP/2 API surface.
//!
//! Reads (`/search`, `/duplicates`, `/stats`) can be served by any
//! replica. Writes (`PUT /collections/...`, `POST .../index`,
//! `DELETE .../index/...`) currently target the local in-memory
//! [`Engine`]; when Raft is wired in they will be forwarded to the
//! shard leader before being applied.
//!
//! The contract for external consumers is `GET /openapi.json`,
//! generated at runtime from this module.

use std::collections::BTreeSet;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{Extension, FromRequest, Path, Query, Request, State},
    http::{Method, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use futures::future::join_all;
use serde::Deserialize;
use service_http::{MetricsProvider, ReadinessHook};
use utoipa::{
    openapi::{
        self,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
    Modify, OpenApi,
};

use axum::http::HeaderMap;

use crate::auth::{auth_middleware, AuthConfig, AuthContext, LumenVerifier, Role};
use crate::backup_sink::{BackupSink, LocalFsSink};
use crate::coordinator::{SubmitStalled, WriteCoordinator, WriteSink};
use crate::log_entry::RaftLogEntry;
use crate::raft::{ClusterStateView, RaftRole, ReadConsistency};
use crate::reshard::ReshardBatch;
use crate::routing::VirtualBucketShardMap;
use crate::storage::{ApplyOutcome, DropOutcome, Engine, SnapshotV1, StorageError};
use crate::types::{
    Analyzer, ApiError, BatchSearchRequest, BatchSearchResponse, BatchSearchResult, CacheStats,
    CreateCollectionRequest, CreateCollectionResponse, DuplicateGroup, DuplicatesRequest,
    DuplicatesResponse, FieldSpec, FieldStats, FieldType, FieldValue, IndexItem, IndexRequest,
    IndexResponse, KnnQuery, MatchOp, MatchQuery, QueryNode, RangeQuery, ReplaceDocBody,
    ReplaceDocItem, ReplaceDocResult, ReplaceDocsRequest, ReplaceDocsResponse, SearchHit,
    SearchRequest, SearchResponse, StatsResponse, StorageStats, TermQuery, TermsQuery,
    VectorBackend, VectorMetric, VectorQuantize, VectorSpec, MAX_BATCH_REPLACE_SIZE,
    MAX_BATCH_SEARCH_SIZE,
};
use crate::wal::{MemWal, SharedWal};

#[derive(Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub struct AppState {
    pub engine: Arc<Engine>,
    pub auth: Arc<AuthConfig>,
    pub cluster: Option<Arc<crate::raft::ClusterState>>,
    /// Read/search backend. Defaults to the local engine; sharded serving can
    /// replace it with a fan-in router while keeping writes/stats local.
    pub search_backend: Arc<dyn SearchBackend>,
    /// Writes go through a [`WriteSink`]: the WAL-seam coordinator for
    /// embedded/nats, or the raft host for `--wal raft`. Reads use
    /// `engine` directly. See `coordinator` / `wal` / `raft_sm`.
    pub writer: Arc<dyn WriteSink>,
    /// Write/mutation backend. Defaults to the local coordinator; sharded
    /// serving can replace it with a document-router that fans out writes
    /// across independent shard coordinators.
    pub write_backend: Arc<dyn WriteBackend>,
    /// Durability-on-demand seam for `POST /admin/checkpoint` (#1389).
    /// Defaults to [`NoopCheckpoint`]; the server binary wires a real
    /// segment-checkpoint implementation when segment persistence is
    /// configured. See [`CheckpointSink`].
    pub checkpoint: Arc<dyn CheckpointSink>,
    /// Bounded write pause on still-moving virtual buckets during a
    /// reshard's final `CatchingUp` pass (#1396 R2). Defaults to unarmed
    /// (every write passes through unchanged); the reshard driver arms it
    /// via `POST /admin/reshard:fence`. See [`WriteFence`].
    pub write_fence: WriteFence,
    /// Cross-pod shard router for operator/k8s serving (#1398 R1-R3).
    /// `None` for every deployment shape except the routed one (`SHARD_COUNT`
    /// > 1, `replicasPerShard <= 1`, no `--search-shard-segment-dirs`) — see
    /// [`RoutedBackend`]. The server binary wires a real
    /// `routing_remote::RoutedRouter` via [`Self::with_routed`]; tests and
    /// every other deployment shape leave this `None`.
    pub routed: Option<Arc<dyn RoutedBackend>>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub trait SearchBackend: Send + Sync {
    fn search(&self, collection_id: &str, req: SearchRequest) -> Result<SearchResponse>;
}

/// Forces a synchronous, awaited durability checkpoint of the live engine
/// state (#1389). The reshard driver's cutover (`operator::reshard_driver::
/// advance_catching_up`) calls `POST /admin/checkpoint` — which routes here —
/// on every shard it just migrated data into or evicted data from, and waits
/// for the response before flipping `spec.shardMap` and triggering the
/// cutover rolling restart. `Engine::apply_reshard_batch`/`evict_not_owned`
/// (`storage.rs`, #1380) mutate engine state directly rather than through
/// `WriteCoordinator`/the AOF, so — unlike ordinary writes — their durability
/// is not implied by `applied_seq()`; this seam is what makes it durable
/// on-demand instead of only on the next periodic `LUMEN_SNAPSHOT_SECS` tick.
///
/// [`NoopCheckpoint`] is the default (no `--data-dir`/non-segment-persistence
/// deployments, including every existing test `AppState`): `checkpoint_now`
/// trivially returns `Ok(false)` (nothing configured to persist, so nothing
/// to lose across an in-process test's non-restart). The server binary wires
/// a real segment-checkpoint-backed implementation whenever
/// `--persistence=segment` + `--data-dir` are configured — exactly the
/// combination the operator now renders unconditionally at
/// `replicasPerShard <= 1` (#1387), which is the same topology the reshard
/// driver is scoped to (see `reshard_driver`'s "Scope rail" doc).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
#[async_trait]
pub trait CheckpointSink: Send + Sync {
    /// Persist current engine state durably and return only once the write
    /// is committed. `Ok(true)` when a checkpoint was actually written;
    /// `Ok(false)` when no durable store is configured (a checkpoint request
    /// against such a deployment is vacuously satisfied — there is nothing
    /// on disk to fall behind). `Err` on a real write failure, which callers
    /// (the reshard driver) must treat as "not yet durable" and retry.
    async fn checkpoint_now(&self) -> Result<bool>;
}

/// Default [`CheckpointSink`] for deployments/tests with no configured
/// durable store — see the trait doc.
struct NoopCheckpoint;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
#[async_trait]
impl CheckpointSink for NoopCheckpoint {
    async fn checkpoint_now(&self) -> Result<bool> {
        Ok(false)
    }
}

/// A bounded, status-visible write pause on a set of still-moving virtual
/// buckets (#1396 R2), the mechanism #1381's R5 review sanctioned: "a
/// bounded final pause of writes to still-moving buckets is acceptable if
/// needed for convergence, but must be bounded and reported in status."
///
/// Closes the copy-to-evict gap in the reshard driver's `CatchingUp` pass:
/// without a pause, a write landing on a source shard's moving bucket after
/// the last migration-copy read but before that bucket's eviction is never
/// re-copied to the target and is silently dropped by eviction. The driver
/// arms a fence over exactly the buckets its final migration pass is about
/// to copy (`POST /admin/reshard:fence`) immediately before that pass, so
/// the single pass taken under the fence is already a complete/converged
/// snapshot of those buckets — no repeat-until-converged loop is needed —
/// and clears the fence (an empty-`buckets` call to the same verb) on every
/// exit path of that tick, success or `Blocked`.
///
/// Crash safety: a driver process that dies between arming and clearing
/// cannot leave a bucket permanently unwritable. [`WriteFence::blocks`]
/// checks `deadline` on every call and treats an expired fence as unarmed —
/// this check runs on the *serving pod*, independent of whether the driver
/// process that armed it is still alive, so expiry is enforced even if the
/// driver never comes back. The reshard driver re-arms a fresh deadline
/// every tick it needs one, so a healthy, slow-but-progressing driver never
/// races its own TTL; see `reshard_driver::WRITE_FENCE_TTL`.
#[derive(Clone, Default)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub struct WriteFence {
    state: Arc<Mutex<Option<FenceState>>>,
}

struct FenceState {
    virtual_bucket_count: u32,
    buckets: BTreeSet<u32>,
    deadline: Instant,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl WriteFence {
    /// Arm the fence over `buckets` (computed against `virtual_bucket_count`)
    /// until `ttl` from now, replacing any prior armed state. Returns `false`
    /// (leaving any prior armed state untouched) when `Instant::now() + ttl`
    /// would overflow (#1443 R3) — the caller must treat that as a failed arm
    /// rather than silently panicking with the fence lock held, which would
    /// poison it for every subsequent write/clear on this pod.
    fn arm(&self, virtual_bucket_count: u32, buckets: BTreeSet<u32>, ttl: Duration) -> bool {
        let Some(deadline) = Instant::now().checked_add(ttl) else {
            return false;
        };
        let mut guard = self.lock();
        *guard = Some(FenceState {
            virtual_bucket_count,
            buckets,
            deadline,
        });
        true
    }

    /// Explicitly disarm, independent of `deadline`.
    fn clear(&self) {
        *self.lock() = None;
    }

    /// `Some(bucket)` when `collection_id`/`external_id` route to a
    /// currently-fenced bucket; `None` (unblocked) once armed but past
    /// `deadline`, or never armed at all.
    fn blocks(&self, collection_id: &str, external_id: &str) -> Option<u32> {
        let guard = self.lock();
        let fence = guard.as_ref()?;
        if Instant::now() >= fence.deadline {
            return None;
        }
        let map = VirtualBucketShardMap::balanced(0, fence.virtual_bucket_count, 1).ok()?;
        let bucket = map.route_document(collection_id, None, external_id).bucket;
        fence.buckets.contains(&bucket).then_some(bucket)
    }

    /// Poison-proof lock acquisition (#1443 R3), matching `segment_rdb.rs`'s
    /// `save_lock` precedent: a panic anywhere else in the process while
    /// holding this lock must never turn into a permanent write outage on
    /// this pod by propagating a poisoned-mutex panic into every later
    /// `arm`/`clear`/`blocks` call.
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<FenceState>> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub trait WriteBackend: Send + Sync {
    async fn create_collection(
        &self,
        collection_id: String,
        req: CreateCollectionRequest,
    ) -> Result<CreateCollectionResponse>;

    async fn drop_collection(&self, collection_id: String, force: bool) -> Result<DropOutcome>;

    async fn index(&self, collection_id: String, req: IndexRequest) -> Result<IndexResponse>;

    async fn replace_docs(
        &self,
        collection_id: String,
        req: ReplaceDocsRequest,
    ) -> Result<ReplaceDocsResponse>;

    async fn delete(
        &self,
        collection_id: String,
        external_id: String,
        field: Option<String>,
    ) -> Result<()>;

    async fn drop_field(&self, collection_id: String, field_name: String) -> Result<u32>;
}

/// Cross-pod shard routing for operator/k8s serving pods (#1398 R1-R3).
/// `AppState::routed` is `None` for every non-routed deployment (standalone,
/// primary/replica, and the `--search-shard-segment-dirs` fan-in path) —
/// handlers consult it first and fall back to `search_backend`/
/// `write_backend` unchanged when it is absent, so `shardCount:1` serving
/// never even constructs an implementation (AC5: no forwarding overhead).
///
/// Every method takes the inbound request's `headers` verbatim: the sole
/// concrete implementation ([`crate::routing_remote::RoutedRouter`], behind
/// the `operator` feature) checks the `x-lumen-forwarded` one-hop guard
/// first and, when forwarding, carries the caller's `Authorization` bearer
/// and `x-read-consistency` through unchanged (R3).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
#[async_trait]
pub trait RoutedBackend: Send + Sync {
    async fn search(
        &self,
        collection_id: &str,
        req: SearchRequest,
        headers: &HeaderMap,
    ) -> Result<SearchResponse>;

    async fn index(
        &self,
        collection_id: String,
        req: IndexRequest,
        headers: &HeaderMap,
    ) -> Result<IndexResponse>;

    async fn replace_docs(
        &self,
        collection_id: String,
        req: ReplaceDocsRequest,
        headers: &HeaderMap,
    ) -> Result<ReplaceDocsResponse>;

    async fn delete(
        &self,
        collection_id: String,
        external_id: String,
        field: Option<String>,
        headers: &HeaderMap,
    ) -> Result<()>;
}

#[derive(Clone)]
struct LocalEngineSearch {
    engine: Arc<Engine>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl SearchBackend for LocalEngineSearch {
    fn search(&self, collection_id: &str, req: SearchRequest) -> Result<SearchResponse> {
        self.engine.search(collection_id, req)
    }
}

#[derive(Clone)]
struct LocalWriteBackend {
    writer: Arc<dyn WriteSink>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl LocalWriteBackend {
    fn unexpected(outcome: ApplyOutcome) -> anyhow::Error {
        anyhow::anyhow!("unexpected apply outcome: {outcome:?}")
    }
}

#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl WriteBackend for LocalWriteBackend {
    async fn create_collection(
        &self,
        collection_id: String,
        req: CreateCollectionRequest,
    ) -> Result<CreateCollectionResponse> {
        match self
            .writer
            .submit(RaftLogEntry::CreateCollection { collection_id, req })
            .await?
        {
            ApplyOutcome::Created(r) => Ok(r),
            other => Err(Self::unexpected(other)),
        }
    }

    async fn drop_collection(&self, collection_id: String, force: bool) -> Result<DropOutcome> {
        match self
            .writer
            .submit(RaftLogEntry::DropCollection {
                collection_id,
                force,
            })
            .await?
        {
            ApplyOutcome::Dropped(o) => Ok(o),
            other => Err(Self::unexpected(other)),
        }
    }

    async fn index(&self, collection_id: String, req: IndexRequest) -> Result<IndexResponse> {
        match self
            .writer
            .submit(RaftLogEntry::Index { collection_id, req })
            .await?
        {
            ApplyOutcome::Indexed(r) => Ok(r),
            other => Err(Self::unexpected(other)),
        }
    }

    async fn replace_docs(
        &self,
        collection_id: String,
        req: ReplaceDocsRequest,
    ) -> Result<ReplaceDocsResponse> {
        match self
            .writer
            .submit(RaftLogEntry::ReplaceDocs { collection_id, req })
            .await?
        {
            ApplyOutcome::Replaced(r) => Ok(r),
            other => Err(Self::unexpected(other)),
        }
    }

    async fn delete(
        &self,
        collection_id: String,
        external_id: String,
        field: Option<String>,
    ) -> Result<()> {
        match self
            .writer
            .submit(RaftLogEntry::Delete {
                collection_id,
                external_id,
                field,
            })
            .await?
        {
            ApplyOutcome::Deleted => Ok(()),
            other => Err(Self::unexpected(other)),
        }
    }

    async fn drop_field(&self, collection_id: String, field_name: String) -> Result<u32> {
        match self
            .writer
            .submit(RaftLogEntry::DropField {
                collection_id,
                field_name,
            })
            .await?
        {
            ApplyOutcome::FieldChanged(v) => Ok(v),
            other => Err(Self::unexpected(other)),
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl AppState {
    /// Build state with an explicit write log. Spawns the apply loop.
    pub fn with_wal(engine: Arc<Engine>, auth: Arc<AuthConfig>, wal: SharedWal) -> Self {
        let writer = WriteCoordinator::start(wal, engine.clone());
        Self::with_components(engine, auth, writer)
    }

    /// Build state from an already-constructed coordinator — used by the
    /// server binary, which wires the WAL + RDB bootstrap itself and
    /// hands in the resulting coordinator.
    pub fn with_components(
        engine: Arc<Engine>,
        auth: Arc<AuthConfig>,
        writer: Arc<dyn WriteSink>,
    ) -> Self {
        Self {
            search_backend: Arc::new(LocalEngineSearch {
                engine: engine.clone(),
            }),
            write_backend: Arc::new(LocalWriteBackend {
                writer: writer.clone(),
            }),
            engine,
            auth,
            cluster: None,
            writer,
            checkpoint: Arc::new(NoopCheckpoint),
            write_fence: WriteFence::default(),
            routed: None,
        }
    }

    /// Build state with an in-process [`MemWal`] — single-node /
    /// dev / tests. Writes feel synchronous.
    pub fn new(engine: Arc<Engine>, auth: Arc<AuthConfig>) -> Self {
        Self::with_wal(engine, auth, Arc::new(MemWal::new()))
    }

    pub fn with_cluster(mut self, cluster: Arc<crate::raft::ClusterState>) -> Self {
        self.cluster = Some(cluster);
        self
    }

    pub fn with_search_backend(mut self, search_backend: Arc<dyn SearchBackend>) -> Self {
        self.search_backend = search_backend;
        self
    }

    pub fn with_write_backend(mut self, write_backend: Arc<dyn WriteBackend>) -> Self {
        self.write_backend = write_backend;
        self
    }

    /// Wire a real [`CheckpointSink`] (#1389) — used by the server binary
    /// when segment persistence is configured, and by tests that need to
    /// control/observe `POST /admin/checkpoint` behavior.
    pub fn with_checkpoint(mut self, checkpoint: Arc<dyn CheckpointSink>) -> Self {
        self.checkpoint = checkpoint;
        self
    }

    /// Wire a [`RoutedBackend`] (#1398) — the server binary calls this only
    /// in the routed serving topology (`SHARD_COUNT` env > 1 at
    /// `replicasPerShard <= 1`, no `--search-shard-segment-dirs`); every
    /// other deployment shape leaves `routed` at its `None` default.
    pub fn with_routed(mut self, routed: Arc<dyn RoutedBackend>) -> Self {
        self.routed = Some(routed);
        self
    }

    /// No-auth state over an in-process log. Used by tests and the
    /// simplest single-node runs.
    pub fn open(engine: Arc<Engine>) -> Self {
        Self::with_wal(
            engine,
            Arc::new(AuthConfig::open()),
            Arc::new(MemWal::new()),
        )
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "lumen",
        description = "Standalone search and duplicate-detection index. Generic Collection / Field primitive; the caller owns the source of truth.",
        license(name = "MIT")
    ),
    servers(
        (url = "http://lumen-svc:7373", description = "in-cluster ClusterIP"),
        (url = "http://localhost:7373", description = "local dev")
    ),
    tags(
        (name = "Collections", description = "Schema lifecycle"),
        (name = "Index",       description = "Document writes & deletes"),
        (name = "Query",       description = "Search & duplicate detection"),
        (name = "Admin",       description = "Health, stats, OpenAPI")
    ),
    paths(
        healthz,
        readyz,
        version,
        metrics,
        debug_cluster,
        list_collections,
        create_collection,
        drop_collection,
        drop_field,
        index,
        delete_external_id,
        replace_docs,
        replace_doc,
        search,
        batch_search,
        duplicates,
        stats,
        backup_scoped,
        reshard_apply,
        reshard_prune,
        reshard_evict,
        reshard_fence,
        admin_checkpoint,
    ),
    components(schemas(
        CreateCollectionRequest,
        CreateCollectionResponse,
        FieldSpec,
        FieldType,
        Analyzer,
        VectorSpec,
        VectorMetric,
        VectorBackend,
        VectorQuantize,
        IndexRequest,
        IndexItem,
        FieldValue,
        IndexResponse,
        ReplaceDocsRequest,
        ReplaceDocItem,
        ReplaceDocsResponse,
        ReplaceDocResult,
        ReplaceDocBody,
        SearchRequest,
        QueryNode,
        MatchQuery,
        MatchOp,
        TermQuery,
        TermsQuery,
        RangeQuery,
        // #1307: $ref'd by RangeQuery's gt/gte/lt/lte bounds (untagged f64 | String) —
        // same dangling-ref reason as the #200 note below, registered explicitly.
        crate::types::RangeBound,
        KnnQuery,
        crate::types::RrfQuery,
        crate::types::ExistsQuery,
        crate::types::DuplicatedQuery,
        // #200: these are $ref'd by QueryNode / SearchRequest but were not
        // registered, so the emitted OpenAPI had dangling refs. SortSpec also
        // pulls in SortOrder + SortMissing.
        crate::types::IdsQuery,
        crate::types::HasChildQuery,
        crate::types::HammingQuery,
        crate::types::SortSpec,
        crate::types::SortOrder,
        crate::types::SortMissing,
        SearchHit,
        SearchResponse,
        BatchSearchRequest,
        crate::types::BatchSearchItem,
        BatchSearchResponse,
        BatchSearchResult,
        DuplicatesRequest,
        DuplicateGroup,
        DuplicatesResponse,
        StatsResponse,
        FieldStats,
        StorageStats,
        CacheStats,
        ApiError,
        crate::raft::ClusterStateView,
        crate::raft::PeerAddr,
        crate::raft::RaftRole,
    )),
    modifiers(&SecurityAddon),
    security(("bearerAuth" = []))
)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub struct ApiDoc;

struct SecurityAddon;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("opaque")
                        .description(Some(
                            "Send `Authorization: Bearer <LUMEN_TOKEN>` when `LUMEN_AUTH=required`.",
                        ))
                        .build(),
                ),
            );
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl ReadinessHook for Engine {
    fn is_draining(&self) -> bool {
        Engine::is_draining(self)
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl MetricsProvider for Engine {
    fn render_metrics(&self) -> String {
        self.metrics().render()
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub fn router(state: AppState) -> Router {
    // Apply auth middleware only to data-plane routes. Admin/Probe
    // endpoints (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
    // `/docs`) stay open so K8s probes and Prometheus scrape can hit
    // them without a token even when auth is required.
    let auth_state = Arc::new(LumenVerifier::new(state.auth.clone()));
    let data_plane = Router::new()
        .route(
            "/collections",
            get(list_collections)
                .options(collections_query_probe)
                .head(collections_query_probe)
                // Epic #1296 R1: `QUERY /collections` is a dual-registered
                // twin of `POST /collections:search` (#1271 batch search).
                // Axum has no native `Method::QUERY` support yet
                // (tokio-rs/axum#3799, PR #3801 open), so this is the interim
                // dispatch — `fallback` runs for any method not explicitly
                // registered above (`GET`, `OPTIONS`, `HEAD`), and the
                // handler re-checks by hand. Replace with a native
                // `MethodFilter::QUERY` combinator once that PR lands.
                .fallback(collections_query_dispatch),
        )
        .route(
            "/collections/{collection_id}",
            put(create_collection)
                .delete(drop_collection)
                .options(collection_id_query_probe)
                .head(collection_id_query_probe)
                // Epic #1296 R1: `QUERY /collections/{collection_id}` is a
                // dual-registered twin of `POST
                // /collections/{collection_id}/search`. See the
                // `/collections` route above for the interim-fallback
                // rationale.
                .fallback(collection_id_query_dispatch),
        )
        .route("/collections/{collection_id}/index", post(index))
        .route(
            "/collections/{collection_id}/index/{external_id}",
            delete(delete_external_id),
        )
        .route(
            "/collections/{collection_id}/docs:replace",
            put(replace_docs),
        )
        .route(
            "/collections/{collection_id}/docs/{external_id}",
            put(replace_doc),
        )
        .route("/collections/{collection_id}/search", post(search))
        .route("/collections:search", post(batch_search))
        .route("/collections/{collection_id}/duplicates", post(duplicates))
        .route("/collections/{collection_id}/stats", get(stats))
        .route(
            "/collections/{collection_id}/fields/{field_name}",
            delete(drop_field),
        )
        .route(
            "/collections/{collection_id}/reindex/stream",
            post(reindex_stream),
        )
        .route("/admin/backup", get(backup))
        .route("/admin/backup/local", post(backup_to_local))
        .route("/admin/backup:scoped", post(backup_scoped))
        .route("/admin/restore", post(restore))
        .route("/admin/reshard:apply", post(reshard_apply))
        .route("/admin/reshard:prune", post(reshard_prune))
        .route("/admin/reshard:evict", post(reshard_evict))
        .route("/admin/reshard:fence", post(reshard_fence))
        .route("/admin/checkpoint", post(admin_checkpoint))
        .layer(from_fn_with_state(auth_state, auth_middleware))
        // Bound request bodies: a bulk index is ~MBs (the item cap is the real
        // guard); 8MiB is the broker payload budget. Rejects oversized
        // bodies with 413 before they hit a handler. Shared with
        // `crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES` (#1444 R2) so the
        // reshard driver's oversize-batch detection can never drift from the
        // limit actually enforced here.
        .layer(axum::extract::DefaultBodyLimit::max(
            crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES,
        ));

    let metrics: Arc<dyn MetricsProvider> = state.engine.clone();
    let probes = service_http::standard_probe_routes(state.engine.clone(), Some(metrics), openapi);
    let admin = Router::new()
        .route("/version", get(version))
        .route("/debug/cluster", get(debug_cluster));

    probes
        .merge(admin.with_state(state.clone()))
        .merge(data_plane.with_state(state))
        // One tracing span per HTTP request — structured request logs always, and
        // the source spans the OTLP layer exports as traces when LUMEN_OTLP_ENDPOINT
        // is set. INFO level so the default `info` EnvFilter keeps it.
        .layer(service_http::trace_layer())
}

#[utoipa::path(
    get,
    path = "/metrics",
    tag = "Admin",
    security(()),
    responses((status = 200, description = "Prometheus text-format metrics", body = String))
)]
/// OpenAPI metadata for the shared `/metrics` implementation in service-http.
#[allow(dead_code)]
async fn metrics(
    State(state): State<AppState>,
) -> (StatusCode, [(&'static str, &'static str); 1], String) {
    let body = state.engine.metrics().render();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        body,
    )
}

#[utoipa::path(
    get,
    path = "/debug/cluster",
    tag = "Admin",
    security(()),
    responses((status = 200, description = "Cluster state snapshot", body = ClusterStateView))
)]
async fn debug_cluster(State(state): State<AppState>) -> Json<ClusterStateView> {
    let view = match state.cluster.as_ref() {
        Some(c) => c.snapshot(),
        None => ClusterStateView {
            pod_name: "local".into(),
            shard_index: 0,
            replica_index: 0,
            role: crate::raft::RaftRole::Leader,
            peers: vec![],
            applied_index: 0,
            leader_term: 0,
            replication_lag_ms: 0,
        },
    };
    Json(view)
}

fn read_consistency_from(headers: &HeaderMap) -> ReadConsistency {
    ReadConsistency::from_header(
        headers
            .get("x-read-consistency")
            .and_then(|h| h.to_str().ok()),
    )
}

/// Enforces a resolved `x-read-consistency` against this pod's live
/// per-shard cluster state (`AppState::cluster`) before a read reaches the
/// local engine (#1310).
///
/// Standalone and legacy external-log builds (`state.cluster` is `None`)
/// have exactly one authoritative copy per shard, so every consistency
/// level is trivially satisfied there — this is a no-op, matching today's
/// behavior unchanged. Primary-replica mode (`state.cluster` is `Some`) is
/// the only place a request's resolved [`ReadConsistency`] can actually
/// diverge from what gets served:
/// - [`ReadConsistency::Any`] is unconstrained.
/// - [`ReadConsistency::Leader`] only succeeds on the pod that currently
///   holds `RaftRole::Leader` for this shard; lumen has no read-forwarding
///   surface, so a non-leader replica rejects the request rather than
///   silently serving a possibly-stale local copy.
/// - [`ReadConsistency::Bounded`] succeeds on the leader (never stale) or
///   on a follower/learner whose `replication_lag_ms` is at or under the
///   requested bound; a replica over the bound rejects rather than
///   silently serving a stale read. In `lumen serve --wal raft`, a
///   follower/learner's `replication_lag_ms` is the conservative "unknown"
///   sentinel (`u64::MAX`) — `RaftHost` doesn't expose a peer-timing RPC
///   today, so `Bounded` on a non-leader replica always rejects rather than
///   report a fabricated lag figure (see `spawn_cluster_state_poller` in
///   `src/bin/lumen.rs`, #1349).
fn enforce_read_consistency(state: &AppState, consistency: ReadConsistency) -> Result<(), ApiErr> {
    let Some(cluster) = state.cluster.as_ref() else {
        return Ok(());
    };
    match consistency {
        ReadConsistency::Any => Ok(()),
        ReadConsistency::Leader => {
            if cluster.role() == RaftRole::Leader {
                return Ok(());
            }
            Err(match cluster.leader_peer() {
                Some(leader) => ApiErr::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "read_consistency_not_leader",
                    format!(
                        "replica `{}` is not the shard {} leader (current leader is `{}`); \
                         leader-consistency reads must reach it",
                        cluster.pod_name, cluster.shard_index, leader.pod_name
                    ),
                ),
                None => ApiErr::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "read_consistency_no_leader",
                    format!(
                        "shard {} has no reachable leader; leader-consistency reads cannot be satisfied",
                        cluster.shard_index
                    ),
                ),
            })
        }
        ReadConsistency::Bounded(bound_ms) => {
            if cluster.role() == RaftRole::Leader {
                return Ok(());
            }
            let lag_ms = cluster.replication_lag_ms.load(Ordering::Relaxed);
            if lag_ms <= bound_ms {
                Ok(())
            } else {
                Err(ApiErr::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "read_consistency_lag_exceeded",
                    format!(
                        "replica `{}` lag {lag_ms}ms exceeds bounded({bound_ms}ms) consistency",
                        cluster.pod_name
                    ),
                ))
            }
        }
    }
}

/// Reject a write whose `(collection_id, external_id)` routes to a
/// currently-fenced virtual bucket (#1396 R2). Checked in [`index`],
/// [`replace_docs`], [`replace_doc`], and [`delete_external_id`] — every
/// write path a reshard's final migration pass must observe a converged
/// snapshot of.
///
/// `delete_external_id` is fenced too (#1458 R2): an earlier revision left
/// DELETE exempt on the theory that `apply_reshard_batch`'s
/// authoritative-subset `replace_ids` scoping (see
/// [`crate::reshard::snapshot_reshard_batches`]'s `replace_mode` and
/// [`crate::storage::Engine::apply_reshard_batch`]'s `replace` parameter)
/// already closes the resurrection gap for a delete acked *before* the
/// final pass's scoped-backup read. That leaves a delete racing strictly
/// inside the sub-window between that read and the same pass's eviction
/// uncovered — fencing DELETE like every other write closes it fully, at
/// the ordinary cost (a retryable 503) of any write to a fenced bucket. See
/// the module's #1396 R2 write-fence doc on [`WriteFence`].
fn enforce_write_fence(
    state: &AppState,
    collection_id: &str,
    external_id: &str,
) -> Result<(), ApiErr> {
    if let Some(bucket) = state.write_fence.blocks(collection_id, external_id) {
        return Err(ApiErr::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "bucket_write_paused",
            format!(
                "virtual bucket {bucket} is paused for an in-progress reshard cutover; retry shortly"
            ),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/healthz",
    tag = "Admin",
    security(()),
    responses((status = 200, description = "Process is alive", body = String))
)]
/// OpenAPI metadata for the shared `/healthz` implementation in service-http.
#[allow(dead_code)]
async fn healthz() -> &'static str {
    "ok"
}

#[utoipa::path(
    get,
    path = "/version",
    tag = "Admin",
    security(()),
    responses((status = 200, description = "Build provenance: version, git sha, build time", body = serde_json::Value))
)]
/// Build provenance. `version` is the crate version; `git_sha` and `built_at`
/// are stamped by `build.rs` and degrade to "unknown" outside a git checkout.
async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "git_sha": option_env!("LUMEN_GIT_SHA").unwrap_or("unknown"),
        "built_at": option_env!("LUMEN_BUILT_AT").unwrap_or("unknown"),
    }))
}

#[utoipa::path(
    get,
    path = "/readyz",
    tag = "Admin",
    security(()),
    responses(
        (status = 200, description = "Engine ready"),
        (status = 503, description = "Not ready")
    )
)]
/// OpenAPI metadata for the shared `/readyz` implementation in service-http.
#[allow(dead_code)]
async fn readyz(State(state): State<AppState>) -> (StatusCode, &'static str) {
    if state.engine.is_draining() {
        (StatusCode::SERVICE_UNAVAILABLE, "draining")
    } else {
        (StatusCode::OK, "ok")
    }
}

// ---------------------------------------------------------------------------
// Collections
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/collections",
    tag = "Collections",
    responses((status = 200, description = "List collection IDs", body = [String]))
)]
async fn list_collections(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<String>>, ApiErr> {
    let all = state.engine.list_collections().map_err(ApiErr::from)?;
    // Filter to what the caller can actually read.
    let visible = all
        .into_iter()
        .filter(|id| auth.ensure(id, Role::Read).is_ok())
        .collect();
    Ok(Json(visible))
}

#[utoipa::path(
    put,
    path = "/collections/{collection_id}",
    tag = "Collections",
    params(("collection_id" = String, Path, description = "Collection namespace")),
    request_body = CreateCollectionRequest,
    responses(
        (status = 200, description = "Collection created", body = CreateCollectionResponse),
        (status = 400, description = "Invalid schema",     body = ApiError)
    )
)]
async fn create_collection(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection_id): Path<String>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<Json<CreateCollectionResponse>, ApiErr> {
    auth.ensure(&collection_id, Role::Admin)?;
    let resp = state
        .write_backend
        .create_collection(collection_id.clone(), req)
        .await
        .map_err(ApiErr::from)?;
    tracing::info!(
        target: "lumen.audit",
        event = "collection_create_or_extend",
        subject = auth.subject().unwrap_or("anonymous"),
        collection_id = %collection_id,
        version = resp.version,
        fields = resp.fields_count,
    );
    Ok(Json(resp))
}

#[derive(Debug, Deserialize)]
struct DropQuery {
    #[serde(default)]
    force: bool,
}

#[utoipa::path(
    delete,
    path = "/collections/{collection_id}",
    tag = "Collections",
    params(
        ("collection_id" = String, Path, description = "Collection namespace"),
        ("force" = Option<bool>, Query, description = "Skip the soft-delete grace window")
    ),
    responses(
        (status = 202, description = "Soft-deleted (grace window)"),
        (status = 204, description = "Physically dropped"),
        (status = 404, description = "Unknown collection")
    )
)]
async fn drop_collection(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection_id): Path<String>,
    Query(q): Query<DropQuery>,
) -> Result<StatusCode, ApiErr> {
    auth.ensure(&collection_id, Role::Admin)?;
    let outcome = state
        .write_backend
        .drop_collection(collection_id.clone(), q.force)
        .await
        .map_err(ApiErr::from)?;
    let phase = match outcome {
        DropOutcome::NotFound => {
            return Err(ApiErr::not_found(format!(
                "collection not found: {collection_id}"
            )))
        }
        DropOutcome::Marked => "marked",
        DropOutcome::AlreadyMarked => "already_marked",
        DropOutcome::Physical => "physical",
    };
    tracing::info!(
        target: "lumen.audit",
        event = "collection_drop",
        phase,
        subject = auth.subject().unwrap_or("anonymous"),
        collection_id = %collection_id,
    );
    // Soft-delete returns 202 Accepted so callers can tell it's still
    // in the grace window; physical / already-marked return 204.
    Ok(match outcome {
        DropOutcome::Marked => StatusCode::ACCEPTED,
        _ => StatusCode::NO_CONTENT,
    })
}

// ---------------------------------------------------------------------------
// Index
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/collections/{collection_id}/index",
    tag = "Index",
    params(("collection_id" = String, Path, description = "Collection namespace")),
    request_body = IndexRequest,
    responses(
        (status = 200, description = "Items indexed",     body = IndexResponse),
        (status = 404, description = "Unknown collection", body = ApiError),
        (status = 422, description = "Type mismatch",      body = ApiError)
    )
)]
async fn index(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Path(collection_id): Path<String>,
    Json(req): Json<IndexRequest>,
) -> Result<Json<IndexResponse>, ApiErr> {
    auth.ensure(&collection_id, Role::Write)?;
    for item in &req.items {
        enforce_write_fence(&state, &collection_id, &item.external_id)?;
    }
    let resp = if let Some(router) = &state.routed {
        router
            .index(collection_id.clone(), req, &headers)
            .await
            .map_err(ApiErr::from)?
    } else {
        state
            .write_backend
            .index(collection_id.clone(), req)
            .await
            .map_err(ApiErr::from)?
    };
    Ok(Json(resp))
}

#[derive(Debug, Deserialize)]
struct DeleteQuery {
    field: Option<String>,
}

#[utoipa::path(
    delete,
    path = "/collections/{collection_id}/index/{external_id}",
    tag = "Index",
    params(
        ("collection_id" = String, Path, description = "Collection namespace"),
        ("external_id"   = String, Path, description = "Caller-owned identifier"),
        ("field"         = Option<String>, Query, description = "Restrict deletion to one field")
    ),
    responses((status = 204, description = "Deleted"))
)]
async fn delete_external_id(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Path((collection_id, external_id)): Path<(String, String)>,
    Query(q): Query<DeleteQuery>,
) -> Result<StatusCode, ApiErr> {
    auth.ensure(&collection_id, Role::Write)?;
    enforce_write_fence(&state, &collection_id, &external_id)?;
    if let Some(router) = &state.routed {
        router
            .delete(collection_id.clone(), external_id, q.field, &headers)
            .await
            .map_err(ApiErr::from)?;
    } else {
        state
            .write_backend
            .delete(collection_id.clone(), external_id, q.field)
            .await
            .map_err(ApiErr::from)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Batch full-replacement upsert: each item's `fields` becomes the doc's
/// entire indexed state, implicitly deleting any declared schema field the
/// doc has today but that is absent from `fields`. `docs:replace` is one
/// literal path segment (AIP-136 custom-method syntax) appended after
/// `{collection_id}`, so it registers directly in axum next to
/// `/collections/{collection_id}/docs/{external_id}` without any capture
/// ambiguity — collection ids are validated to reject `:`.
///
/// PUT is deliberate: this is idempotent full replacement (plus optional
/// doc-level last-write-wins), so replaying the same request converges to
/// the same state. Own the *complete* row for a doc? Use `docs:replace`.
/// Own only *some* fields and want to add/update those without touching
/// the rest? Use `POST .../index` instead.
///
/// One bad item (unknown field, type mismatch) never fails the batch — the
/// batch-level status stays 200 and that item's [`ReplaceDocResult`]
/// carries the error. Only a malformed body or an over-limit batch returns
/// 400.
#[utoipa::path(
    put,
    path = "/collections/{collection_id}/docs:replace",
    tag = "Index",
    params(("collection_id" = String, Path, description = "Collection namespace")),
    request_body = ReplaceDocsRequest,
    responses(
        (status = 200, description = "Per-item results, same order and length as `docs`", body = ReplaceDocsResponse),
        (status = 400, description = "Malformed body or batch size over the limit", body = ApiError)
    )
)]
async fn replace_docs(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Path(collection_id): Path<String>,
    Json(req): Json<ReplaceDocsRequest>,
) -> Result<Json<ReplaceDocsResponse>, ApiErr> {
    auth.ensure(&collection_id, Role::Write)?;
    if req.docs.len() > MAX_BATCH_REPLACE_SIZE {
        return Err(ApiErr::new(
            StatusCode::BAD_REQUEST,
            "batch_too_large",
            format!(
                "batch has {} items, max is {MAX_BATCH_REPLACE_SIZE}",
                req.docs.len()
            ),
        ));
    }
    for doc in &req.docs {
        enforce_write_fence(&state, &collection_id, &doc.external_id)?;
    }
    let resp = replace_docs_routed_or_local(&state, &headers, collection_id, req).await?;
    Ok(Json(resp))
}

/// Shared write-backend/router branch for [`replace_docs`] and
/// [`replace_doc`] (its single-doc sugar).
async fn replace_docs_routed_or_local(
    state: &AppState,
    headers: &HeaderMap,
    collection_id: String,
    req: ReplaceDocsRequest,
) -> Result<ReplaceDocsResponse, ApiErr> {
    if let Some(router) = &state.routed {
        router
            .replace_docs(collection_id, req, headers)
            .await
            .map_err(ApiErr::from)
    } else {
        state
            .write_backend
            .replace_docs(collection_id, req)
            .await
            .map_err(ApiErr::from)
    }
}

/// Single-resource sugar over `docs:replace`: exactly the one-item batch
/// `{"docs": [{"external_id": ..., "version": ..., "fields": {...}}]}`,
/// unwrapped back into a bare [`ReplaceDocResult`]. See [`replace_docs`]
/// for the full-replacement / doc-level LWW semantics — the batch-level
/// status stays 200 here too; a bad item comes back as
/// `{"status":"error",...}` in the body rather than as an HTTP error.
#[utoipa::path(
    put,
    path = "/collections/{collection_id}/docs/{external_id}",
    tag = "Index",
    params(
        ("collection_id" = String, Path, description = "Collection namespace"),
        ("external_id"   = String, Path, description = "Caller-owned identifier")
    ),
    request_body = ReplaceDocBody,
    responses(
        (status = 200, description = "Replacement result for this doc", body = ReplaceDocResult),
        (status = 400, description = "Malformed body", body = ApiError)
    )
)]
async fn replace_doc(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Path((collection_id, external_id)): Path<(String, String)>,
    Json(body): Json<ReplaceDocBody>,
) -> Result<Json<ReplaceDocResult>, ApiErr> {
    auth.ensure(&collection_id, Role::Write)?;
    enforce_write_fence(&state, &collection_id, &external_id)?;
    let req = ReplaceDocsRequest {
        docs: vec![ReplaceDocItem {
            external_id,
            version: body.version,
            fields: body.fields,
        }],
    };
    let resp = replace_docs_routed_or_local(&state, &headers, collection_id, req).await?;
    let result = resp.results.into_iter().next().ok_or_else(|| {
        ApiErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "no result for single-doc replace".to_string(),
        )
    })?;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Query
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/collections/{collection_id}/search",
    tag = "Query",
    params(("collection_id" = String, Path, description = "Collection namespace")),
    request_body = SearchRequest,
    responses((status = 200, description = "Search hits", body = SearchResponse))
)]
async fn search(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Path(collection_id): Path<String>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiErr> {
    Ok(Json(
        search_core(&state, &auth, &headers, &collection_id, req).await?,
    ))
}

/// Shared implementation behind `POST /collections/{collection_id}/search`
/// and its `QUERY /collections/{collection_id}` twin
/// ([`collection_id_query_dispatch`], epic #1296 R1: every QUERY endpoint
/// keeps a POST twin — same handler, identical response). Consults
/// `state.routed` first (#1398 R1) — a routed deployment scatters/forwards
/// by ownership; every other deployment falls through to `search_backend`
/// unchanged.
async fn search_core(
    state: &AppState,
    auth: &AuthContext,
    headers: &HeaderMap,
    collection_id: &str,
    req: SearchRequest,
) -> Result<SearchResponse, ApiErr> {
    auth.ensure(collection_id, Role::Read)?;
    let consistency = read_consistency_from(headers);
    enforce_read_consistency(state, consistency)?;
    if let Some(router) = &state.routed {
        return router
            .search(collection_id, req, headers)
            .await
            .map_err(ApiErr::from);
    }
    state
        .search_backend
        .search(collection_id, req)
        .map_err(ApiErr::from)
}

/// msearch-style batch search: N independent `(collection, SearchRequest)`
/// items in one HTTP request, fanned out concurrently. `collections:search`
/// is one literal path segment (AIP-136 custom-method syntax), so it
/// registers directly in axum next to `/collections` and
/// `/collections/{collection_id}` without any capture ambiguity.
///
/// One item failing (e.g. an unknown collection) never fails the batch —
/// the batch-level status stays 200 and that item's [`BatchSearchResult`]
/// carries the error. Only a malformed body or an over-limit batch returns
/// 400. Cursors, sort, and collapse all stay per-item: there is no merged
/// cursor and no cross-collection score merging.
#[utoipa::path(
    post,
    path = "/collections:search",
    tag = "Query",
    request_body = BatchSearchRequest,
    responses(
        (status = 200, description = "Per-item results, same order and length as `searches`", body = BatchSearchResponse),
        (status = 400, description = "Malformed body or batch size over the limit", body = ApiError)
    )
)]
async fn batch_search(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Json(req): Json<BatchSearchRequest>,
) -> Result<Json<BatchSearchResponse>, ApiErr> {
    Ok(Json(batch_search_core(&state, &auth, &headers, req).await?))
}

/// Shared implementation behind `POST /collections:search` and its `QUERY
/// /collections` twin ([`collections_query_dispatch`], epic #1296 R1: every
/// QUERY endpoint keeps a POST twin — same handler, identical response).
async fn batch_search_core(
    state: &AppState,
    auth: &AuthContext,
    headers: &HeaderMap,
    req: BatchSearchRequest,
) -> Result<BatchSearchResponse, ApiErr> {
    if req.searches.len() > MAX_BATCH_SEARCH_SIZE {
        return Err(ApiErr::new(
            StatusCode::BAD_REQUEST,
            "batch_too_large",
            format!(
                "batch has {} items, max is {MAX_BATCH_SEARCH_SIZE}",
                req.searches.len()
            ),
        ));
    }
    let consistency = read_consistency_from(headers);
    enforce_read_consistency(state, consistency)?;
    let results = join_all(req.searches.into_iter().map(|item| {
        let state = state.clone();
        let auth = auth.clone();
        let headers = headers.clone();
        async move {
            if let Err(e) = auth.ensure(&item.collection, Role::Read) {
                return batch_search_auth_error(e);
            }
            let result = if let Some(router) = &state.routed {
                router
                    .search(&item.collection, item.request, &headers)
                    .await
            } else {
                state.search_backend.search(&item.collection, item.request)
            };
            match result {
                Ok(response) => BatchSearchResult::Ok { response },
                Err(e) => batch_search_storage_error(e),
            }
        }
    }))
    .await;
    Ok(BatchSearchResponse { results })
}

// ---------------------------------------------------------------------------
// QUERY (RFC 10008) — dual-registered POST twins (epic #1296 R1)
// ---------------------------------------------------------------------------
//
// axum has no native `Method::QUERY`/`MethodFilter::QUERY` yet
// (tokio-rs/axum#3799, PR #3801 open). The interim dispatch below registers
// each route's `fallback` — the handler axum calls for any method not
// explicitly claimed by that route's `get`/`post`/`put`/`delete`/`options`/
// `head` combinators — and re-checks the method by hand via
// `Method::from_bytes(b"QUERY")`. Replace `is_query_method` and both
// `*_query_dispatch` fallbacks with native `MethodFilter::QUERY` combinators
// once that PR lands; `*_query_probe` (OPTIONS/HEAD) can move to ordinary
// combinators unchanged.

/// `true` for the RFC 10008 QUERY method. `http::Method` has no `QUERY`
/// constant yet, so this matches the wire token the same way
/// `Method::from_bytes(b"QUERY")` would.
fn is_query_method(method: &Method) -> bool {
    Method::from_bytes(b"QUERY").is_ok_and(|query| *method == query)
}

/// 405 for any method that reaches a QUERY-dispatch fallback without
/// actually being QUERY. Normal traffic never hits this arm — `PUT`/
/// `DELETE`/`GET`/`OPTIONS`/`HEAD` are all claimed by explicit combinators
/// ahead of the fallback — it only guards stray/unsupported methods.
fn query_method_not_allowed(allow: &'static str) -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header(axum::http::header::ALLOW, allow)
        .body(axum::body::Body::empty())
        .expect("static not-allowed headers are always valid")
}

/// `OPTIONS`/`HEAD` probe response shared by both QUERY targets: advertises
/// `Accept-Query: application/json` (RFC 10008 discovery) and lists the
/// target's full method set, QUERY included, in `Allow`.
fn query_probe_response(allow: &'static str) -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(axum::http::header::ALLOW, allow)
        .header("accept-query", "application/json")
        .body(axum::body::Body::empty())
        .expect("static probe headers are always valid")
}

async fn collection_id_query_probe() -> axum::response::Response {
    query_probe_response("PUT, DELETE, QUERY, OPTIONS, HEAD")
}

async fn collections_query_probe() -> axum::response::Response {
    query_probe_response("GET, QUERY, OPTIONS, HEAD")
}

/// `QUERY /collections/{collection_id}` — dual-registered twin of `POST
/// /collections/{collection_id}/search` (same [`search_core`] handler,
/// identical response for identical bodies). Content-Type is mandatory on
/// QUERY per RFC 10008; reusing [`Json`]'s own `FromRequest` for the body
/// gives that for free — missing/mismatched `Content-Type` rejects with 415,
/// byte-identical to what the POST twin already returns for the same input.
async fn collection_id_query_dispatch(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection_id): Path<String>,
    request: Request,
) -> axum::response::Response {
    if !is_query_method(request.method()) {
        return query_method_not_allowed("PUT, DELETE, QUERY, OPTIONS, HEAD");
    }
    let headers = request.headers().clone();
    match Json::<SearchRequest>::from_request(request, &state).await {
        Ok(Json(req)) => match search_core(&state, &auth, &headers, &collection_id, req).await {
            Ok(resp) => Json(resp).into_response(),
            Err(e) => e.into_response(),
        },
        Err(rejection) => rejection.into_response(),
    }
}

/// `QUERY /collections` — dual-registered twin of `POST /collections:search`
/// (same [`batch_search_core`] handler, identical response for identical
/// bodies). See [`collection_id_query_dispatch`] for the Content-Type/415
/// and interim-fallback rationale.
async fn collections_query_dispatch(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    request: Request,
) -> axum::response::Response {
    if !is_query_method(request.method()) {
        return query_method_not_allowed("GET, QUERY, OPTIONS, HEAD");
    }
    let headers = request.headers().clone();
    match Json::<BatchSearchRequest>::from_request(request, &state).await {
        Ok(Json(req)) => match batch_search_core(&state, &auth, &headers, req).await {
            Ok(resp) => Json(resp).into_response(),
            Err(e) => e.into_response(),
        },
        Err(rejection) => rejection.into_response(),
    }
}

/// Classify one batch item's search failure into a
/// [`BatchSearchResult::Error`] instead of failing the whole batch. Mirrors
/// `From<anyhow::Error> for ApiErr`'s `StorageError` classification, but the
/// `code` values line up with the batch wire contract
/// (`"collection_not_found"`, ...) rather than `ApiErr`'s internal `kind`
/// strings.
fn batch_search_storage_error(e: anyhow::Error) -> BatchSearchResult {
    let code = match e.downcast_ref::<StorageError>() {
        Some(StorageError::CollectionNotFound(_)) => "collection_not_found",
        Some(StorageError::InvalidCollectionName(_)) => "invalid_collection_name",
        Some(StorageError::UnknownField { .. }) => "unknown_field",
        Some(StorageError::TypeMismatch { .. }) => "type_mismatch",
        Some(StorageError::DuplicatesOnText(_)) => "bad_request",
        Some(StorageError::InvalidNumber(_)) => "invalid_number",
        Some(StorageError::BulkLimit { .. }) => "bulk_limit",
        Some(StorageError::QueryTooComplex(_)) => "query_too_complex",
        Some(StorageError::Gone(_)) => "gone",
        Some(StorageError::UnsupportedSort(_)) => "unsupported_sort",
        Some(StorageError::InvalidPruneChunk { .. }) => "invalid_prune_chunk",
        Some(StorageError::PruneAccumulatorFull { .. }) => "prune_accumulator_full",
        None => "bad_request",
    };
    BatchSearchResult::Error {
        code: code.to_string(),
        message: e.to_string(),
    }
}

/// Classify one batch item's auth rejection into a
/// [`BatchSearchResult::Error`].
fn batch_search_auth_error(e: crate::auth::AuthErr) -> BatchSearchResult {
    match e {
        crate::auth::AuthErr::Forbidden {
            subject,
            needed,
            collection_id,
        } => BatchSearchResult::Error {
            code: "forbidden".to_string(),
            message: format!("subject `{subject}` lacks {needed:?} on `{collection_id}`"),
        },
    }
}

#[utoipa::path(
    post,
    path = "/collections/{collection_id}/duplicates",
    tag = "Query",
    params(("collection_id" = String, Path, description = "Collection namespace")),
    request_body = DuplicatesRequest,
    responses((status = 200, description = "Duplicate groups", body = DuplicatesResponse))
)]
/// Local-shard only, deliberately not wired to `state.routed` (#1398 known
/// gap, #1442 R6): `Engine::duplicates` filters by `min_group_size` *before*
/// any cross-shard merge could happen, so scatter-then-merge would silently
/// miss a true cross-shard group (e.g. one copy per shard under
/// `min_group_size: 2`) — a correctness regression, not a routing gap. A
/// correct cross-shard implementation needs unfiltered per-shard candidate
/// groups from `storage.rs`, out of scope here. #1442 R6 closes the "silent
/// wrong answer" gap this left in routed multi-shard mode: rather than
/// unchanged pre-#1398 behavior (silently answering from local-shard data
/// only, missing cross-shard duplicate groups with no indication), a routed
/// deployment now rejects with a distinct, non-retryable error so a caller
/// can tell "not supported here" from "no duplicates found".
async fn duplicates(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Path(collection_id): Path<String>,
    Json(req): Json<DuplicatesRequest>,
) -> Result<Json<DuplicatesResponse>, ApiErr> {
    auth.ensure(&collection_id, Role::Read)?;
    let _consistency = read_consistency_from(&headers);
    if state.routed.is_some() {
        return Err(ApiErr::new(
            StatusCode::NOT_IMPLEMENTED,
            "duplicates_not_routed",
            "duplicate detection is local-shard-only and does not merge across shards; not \
             supported in routed multi-shard mode (#1442 R6)"
                .to_string(),
        ));
    }
    Ok(Json(
        state
            .engine
            .duplicates(&collection_id, req)
            .map_err(ApiErr::from)?,
    ))
}

#[utoipa::path(
    get,
    path = "/collections/{collection_id}/stats",
    tag = "Query",
    params(("collection_id" = String, Path, description = "Collection namespace")),
    responses((status = 200, description = "Collection stats", body = StatsResponse))
)]
async fn stats(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection_id): Path<String>,
) -> Result<Json<StatsResponse>, ApiErr> {
    auth.ensure(&collection_id, Role::Read)?;
    Ok(Json(
        state.engine.stats(&collection_id).map_err(ApiErr::from)?,
    ))
}

/// Streaming bulk-reindex endpoint.
///
/// Body is NDJSON of `IndexItem` records (one per line). Response is
/// an NDJSON stream of progress events:
///
/// ```text
/// {"event":"progress","indexed_total":1000,"batch_indexed":1000,"elapsed_ms":42}
/// {"event":"progress","indexed_total":2000,"batch_indexed":1000,"elapsed_ms":85}
/// ...
/// {"event":"done","indexed_total":2473,"elapsed_ms":210}
/// ```
///
/// Errors are surfaced as `{"event":"error","line":N,"message":"..."}`
/// inline; the stream continues so partial progress is observable.
///
/// Rejected outright in routed multi-shard mode (#1442 R6): the spawned
/// batch loop below writes through `state.write_backend` directly, bypassing
/// both `state.routed`'s per-item shard ownership and `enforce_write_fence`
/// (the same per-item reshard-cutover pause every other write path
/// observes). Routing each streamed item by ownership and fencing it
/// individually, inside a detached `tokio::spawn` task that already streams
/// its own NDJSON response back, is a materially bigger change than this
/// bounded hardening pass — an accepted, documented fallback per R6's own
/// scope rather than a half-routed implementation that could silently
/// mis-shard or skip the write fence.
async fn reindex_stream(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection_id): Path<String>,
    body: axum::body::Bytes,
) -> Result<axum::response::Response, ApiErr> {
    use axum::body::Body;
    use std::time::Instant;
    use tokio::sync::mpsc;

    auth.ensure(&collection_id, Role::Write)?;
    if state.routed.is_some() {
        return Err(ApiErr::new(
            StatusCode::NOT_IMPLEMENTED,
            "reindex_stream_not_routed",
            "streaming bulk reindex bypasses per-item shard ownership and the write fence; not \
             supported in routed multi-shard mode, use POST .../index instead (#1442 R6)"
                .to_string(),
        ));
    }

    const BATCH_SIZE: usize = 1_000;
    let (tx, rx) = mpsc::channel::<Result<axum::body::Bytes, std::io::Error>>(16);
    let writer = state.write_backend.clone();
    let collection = collection_id.clone();

    tokio::spawn(async move {
        let started = Instant::now();
        let mut batch: Vec<IndexItem> = Vec::with_capacity(BATCH_SIZE);
        let mut indexed_total = 0u64;
        let send = |tx: &mpsc::Sender<_>, line: serde_json::Value| {
            let mut s = line.to_string();
            s.push('\n');
            let bytes = axum::body::Bytes::from(s.into_bytes());
            tx.try_send(Ok::<_, std::io::Error>(bytes))
        };

        for (lineno, raw) in body.split(|&b| b == b'\n').enumerate() {
            let line = raw.trim_ascii();
            if line.is_empty() {
                continue;
            }
            let item: IndexItem = match serde_json::from_slice(line) {
                Ok(i) => i,
                Err(e) => {
                    let _ = send(
                        &tx,
                        serde_json::json!({
                            "event": "error",
                            "line": lineno + 1,
                            "message": e.to_string(),
                        }),
                    );
                    continue;
                }
            };
            batch.push(item);

            if batch.len() >= BATCH_SIZE {
                let drained = std::mem::replace(&mut batch, Vec::with_capacity(BATCH_SIZE));
                let batch_start = Instant::now();
                match writer
                    .index(
                        collection.clone(),
                        IndexRequest {
                            items: drained,
                            request_id: None,
                        },
                    )
                    .await
                {
                    Ok(r) => {
                        indexed_total += r.indexed as u64;
                        let _ = send(
                            &tx,
                            serde_json::json!({
                                "event": "progress",
                                "indexed_total": indexed_total,
                                "batch_indexed": r.indexed,
                                "elapsed_ms": started.elapsed().as_millis() as u64,
                                "batch_elapsed_ms": batch_start.elapsed().as_millis() as u64,
                            }),
                        );
                    }
                    Err(e) => {
                        let _ = send(
                            &tx,
                            serde_json::json!({
                                "event": "error",
                                "line": lineno + 1,
                                "message": e.to_string(),
                            }),
                        );
                    }
                }
            }
        }

        // Final flush of whatever's left in the batch.
        if !batch.is_empty() {
            let batch_start = Instant::now();
            if let Ok(r) = writer
                .index(
                    collection.clone(),
                    IndexRequest {
                        items: batch,
                        request_id: None,
                    },
                )
                .await
            {
                indexed_total += r.indexed as u64;
                let _ = send(
                    &tx,
                    serde_json::json!({
                        "event": "progress",
                        "indexed_total": indexed_total,
                        "batch_indexed": r.indexed,
                        "elapsed_ms": started.elapsed().as_millis() as u64,
                        "batch_elapsed_ms": batch_start.elapsed().as_millis() as u64,
                    }),
                );
            }
        }

        let _ = send(
            &tx,
            serde_json::json!({
                "event": "done",
                "indexed_total": indexed_total,
                "elapsed_ms": started.elapsed().as_millis() as u64,
            }),
        );

        tracing::info!(
            target: "lumen.audit",
            event = "reindex_stream_done",
            subject = auth.subject().unwrap_or("anonymous"),
            collection_id = %collection,
            indexed_total,
            elapsed_ms = started.elapsed().as_millis() as u64,
        );
    });

    let stream =
        futures::stream::unfold(rx, |mut rx| async move { rx.recv().await.map(|r| (r, rx)) });
    let resp = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/x-ndjson")
        .body(Body::from_stream(stream))
        .map_err(|e| {
            ApiErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "stream_init",
                e.to_string(),
            )
        })?;
    Ok(resp)
}

#[utoipa::path(
    delete,
    path = "/collections/{collection_id}/fields/{field_name}",
    tag = "Collections",
    params(
        ("collection_id" = String, Path, description = "Collection namespace"),
        ("field_name"    = String, Path, description = "Field to drop")
    ),
    responses(
        (status = 200, description = "Field dropped; new schema version", body = serde_json::Value),
        (status = 404, description = "Unknown collection or field",       body = ApiError)
    )
)]
async fn drop_field(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path((collection_id, field_name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    auth.ensure(&collection_id, Role::Admin)?;
    let version = state
        .write_backend
        .drop_field(collection_id.clone(), field_name.clone())
        .await
        .map_err(ApiErr::from)?;
    tracing::info!(
        target: "lumen.audit",
        event = "field_drop",
        subject = auth.subject().unwrap_or("anonymous"),
        collection_id = %collection_id,
        field_name = %field_name,
        version,
    );
    Ok(Json(serde_json::json!({
        "collection_id": collection_id,
        "field_name": field_name,
        "version": version,
    })))
}

// ---------------------------------------------------------------------------
// Backup / restore (cluster-wide admin)
// ---------------------------------------------------------------------------

/// Dump the entire engine state as a single JSON document.
async fn backup(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<SnapshotV1>, ApiErr> {
    // Cluster-wide admin op: needs admin on wildcard.
    auth.ensure("*", Role::Admin)?;
    tracing::info!(
        target: "lumen.audit",
        event = "backup_started",
        subject = auth.subject().unwrap_or("anonymous"),
    );
    Ok(Json(state.engine.snapshot().map_err(ApiErr::from)?))
}

#[derive(Debug, Deserialize)]
struct LocalBackupRequest {
    /// Filesystem path the snapshot will be written into.
    path: String,
    /// Key prefix; the file will be named `{prefix}-{unix_seconds}.json`.
    #[serde(default = "default_backup_prefix")]
    prefix: String,
}

fn default_backup_prefix() -> String {
    "lumen-backup".into()
}

/// Snapshot the engine and persist it via a `LocalFsSink`. Returns the
/// final key the sink chose. The path is created if missing.
async fn backup_to_local(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<LocalBackupRequest>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    auth.ensure("*", Role::Admin)?;
    let snap = state.engine.snapshot().map_err(ApiErr::from)?;
    let payload = serde_json::to_vec(&snap)
        .map_err(|e| ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "encode", e.to_string()))?;
    let sink = LocalFsSink::new(&req.path, &req.prefix)
        .map_err(|e| ApiErr::new(StatusCode::BAD_REQUEST, "bad_sink", e.to_string()))?;
    let key = sink
        .put(std::time::SystemTime::now(), &payload)
        .map_err(|e| ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "sink_put", e.to_string()))?;
    tracing::info!(
        target: "lumen.audit",
        event = "backup_local",
        subject = auth.subject().unwrap_or("anonymous"),
        sink = %sink.identity(),
        key = %key,
        bytes = payload.len(),
    );
    Ok(Json(serde_json::json!({
        "sink": sink.identity(),
        "key": key,
        "bytes": payload.len(),
    })))
}

/// Restore the engine from a snapshot dump produced by `/admin/backup`.
/// Replaces all existing state.
async fn restore(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(snap): Json<SnapshotV1>,
) -> Result<StatusCode, ApiErr> {
    auth.ensure("*", Role::Admin)?;
    state.engine.restore(snap).map_err(ApiErr::from)?;
    tracing::info!(
        target: "lumen.audit",
        event = "restore_applied",
        subject = auth.subject().unwrap_or("anonymous"),
    );
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Reshard admin verbs (#1380): batch-apply, bucket-scoped export, evict.
// Plus `/admin/checkpoint` (#1389), the on-demand durability step that makes
// the other three's mutations survive the cutover restart the driver itself
// triggers, and `/admin/reshard:prune` (#1457 R1), the final migration
// pass's independently chunked authoritative-replace scope.
//
// `reshard.rs`'s tested primitives (`bucket_moves`, `snapshot_reshard_batches`,
// `snapshot_reshard_prune_chunks`) emit bounded `ReshardBatch`/
// `ReshardPruneChunk` units for checkpointed migration; the five verbs below
// are the wire surface that moves them and makes them durable. All five
// require `Role::Admin` on `*`, same as `/admin/backup`/`/admin/restore`
// above.
// ---------------------------------------------------------------------------

/// `POST /admin/reshard:apply`: additively merge one [`ReshardBatch`] into
/// the live engine (upsert semantics for the batch's documents; never a
/// full replace, unlike `/admin/restore`). Idempotent — a retried batch
/// (operator resume after a checkpoint) converges to the same query-visible
/// state; see [`Engine::apply_reshard_batch`].
#[utoipa::path(
    post,
    path = "/admin/reshard:apply",
    tag = "Admin",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Batch merged additively (safe to retry)", body = serde_json::Value),
        (status = 400, description = "Malformed batch or snapshot version mismatch", body = ApiError)
    )
)]
async fn reshard_apply(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(batch): Json<ReshardBatch>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    auth.ensure("*", Role::Admin)?;
    let outcome = state
        .engine
        .apply_reshard_batch(batch.snapshot, None)
        .map_err(ApiErr::from)?;
    tracing::info!(
        target: "lumen.audit",
        event = "reshard_batch_applied",
        subject = auth.subject().unwrap_or("anonymous"),
        bucket = batch.bucket,
        from_shard = batch.from_shard,
        to_shard = batch.to_shard,
        from_map_version = batch.from_map_version,
        to_map_version = batch.to_map_version,
        collections_touched = outcome.collections_touched,
        documents_upserted = outcome.documents_upserted,
        documents_pruned = outcome.documents_pruned,
    );
    Ok(Json(serde_json::json!({
        "collections_touched": outcome.collections_touched,
        "documents_upserted": outcome.documents_upserted,
        "documents_pruned": outcome.documents_pruned,
    })))
}

/// `POST /admin/reshard:prune`: accumulate one [`ReshardPruneChunk`] of the
/// final migration pass's authoritative "keep" set for one `(bucket,
/// collection_id)` pair, and prune once every chunk has arrived (#1457 R1).
/// Unlike `/admin/reshard:apply` (purely additive), this verb is what makes
/// the final pass authoritative for the buckets it copies: a document
/// deleted on the source during the split is absent from the accumulated
/// keep set and is pruned here instead of surviving as a stale copy from an
/// earlier additive pass. Idempotent per chunk (safe to retry after a 413)
/// and as a whole group (safe to re-send every chunk after a driver
/// restart); see [`Engine::apply_reshard_prune_chunk`].
#[utoipa::path(
    post,
    path = "/admin/reshard:prune",
    tag = "Admin",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Chunk accumulated; pruned once every chunk of its group has arrived", body = serde_json::Value),
        (status = 400, description = "Malformed chunk", body = ApiError)
    )
)]
async fn reshard_prune(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(chunk): Json<crate::reshard::ReshardPruneChunk>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    auth.ensure("*", Role::Admin)?;
    let to_map_version = chunk.to_map_version;
    let bucket = chunk.bucket;
    let collection_id = chunk.collection_id.clone();
    let chunk_index = chunk.chunk_index;
    let total_chunks = chunk.total_chunks;
    let outcome = state
        .engine
        .apply_reshard_prune_chunk(chunk)
        .map_err(ApiErr::from)?;
    if outcome.complete {
        tracing::info!(
            target: "lumen.audit",
            event = "reshard_prune_applied",
            subject = auth.subject().unwrap_or("anonymous"),
            bucket,
            to_map_version,
            collection_id = collection_id.as_str(),
            chunk_index,
            total_chunks,
            documents_pruned = outcome.documents_pruned,
        );
    }
    Ok(Json(serde_json::json!({
        "complete": outcome.complete,
        "documents_pruned": outcome.documents_pruned,
    })))
}

#[derive(Debug, Deserialize)]
struct ScopedBackupRequest {
    /// Same `virtual_bucket_count` the caller's [`VirtualBucketShardMap`]
    /// uses — must match what `snapshot_reshard_batches` was/will be called
    /// with so bucket membership agrees.
    virtual_bucket_count: u32,
    /// Only documents whose bucket is in this set are included.
    buckets: BTreeSet<u32>,
}

/// `POST /admin/backup:scoped`: like `GET /admin/backup`, but restricted to
/// documents routed to the requested virtual buckets — a source shard can
/// export just the buckets that are moving instead of a full-engine dump.
/// Bucket membership is computed with the same hash `reshard::
/// snapshot_reshard_batches` uses ([`crate::reshard::snapshot_bucket_subset`]),
/// so an export and a later-computed batch can never disagree.
#[utoipa::path(
    post,
    path = "/admin/backup:scoped",
    tag = "Admin",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "SnapshotV1 restricted to the requested virtual buckets", body = serde_json::Value),
        (status = 400, description = "Invalid virtual_bucket_count", body = ApiError)
    )
)]
async fn backup_scoped(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ScopedBackupRequest>,
) -> Result<Json<SnapshotV1>, ApiErr> {
    auth.ensure("*", Role::Admin)?;
    let full = state.engine.snapshot().map_err(ApiErr::from)?;
    let scoped =
        crate::reshard::snapshot_bucket_subset(&full, req.virtual_bucket_count, &req.buckets)
            .map_err(ApiErr::from)?;
    tracing::info!(
        target: "lumen.audit",
        event = "backup_scoped",
        subject = auth.subject().unwrap_or("anonymous"),
        virtual_bucket_count = req.virtual_bucket_count,
        buckets = req.buckets.len(),
    );
    Ok(Json(scoped))
}

#[derive(Debug, Deserialize)]
struct ReshardEvictRequest {
    /// This shard's physical index in `assignments`.
    shard: u32,
    /// The newer map version being cut over to; carried for audit logging.
    map_version: u64,
    /// `bucket -> physical shard` assignment for the newer map. Its length
    /// is the virtual bucket count.
    assignments: Vec<u32>,
    physical_shard_count: u32,
}

/// `POST /admin/reshard:evict`: source-side post-cutover eviction. Given a
/// newer virtual-bucket map and this shard's index within it, removes
/// exactly the documents whose bucket no longer routes to this shard —
/// nothing else. A separate, explicitly-invoked step; never implicit in
/// `/admin/reshard:apply` or `/admin/backup*`. Idempotent — a document
/// already evicted by a prior call no longer matches and is skipped.
#[utoipa::path(
    post,
    path = "/admin/reshard:evict",
    tag = "Admin",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Documents no longer owned by this shard removed", body = serde_json::Value),
        (status = 400, description = "Invalid virtual bucket map", body = ApiError)
    )
)]
async fn reshard_evict(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ReshardEvictRequest>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    auth.ensure("*", Role::Admin)?;
    let map =
        VirtualBucketShardMap::new(req.map_version, req.assignments, req.physical_shard_count)
            .map_err(ApiErr::from)?;
    let outcome = state
        .engine
        .evict_not_owned(&map, req.shard)
        .map_err(ApiErr::from)?;
    tracing::info!(
        target: "lumen.audit",
        event = "reshard_evict",
        subject = auth.subject().unwrap_or("anonymous"),
        shard = req.shard,
        map_version = req.map_version,
        collections_touched = outcome.collections_touched,
        documents_evicted = outcome.documents_evicted,
    );
    Ok(Json(serde_json::json!({
        "collections_touched": outcome.collections_touched,
        "documents_evicted": outcome.documents_evicted,
    })))
}

/// `POST /admin/checkpoint` (#1389): force a synchronous durability
/// checkpoint of the live engine state and return only once it is committed.
/// The reshard driver's cutover calls this on every shard it just migrated
/// data into or evicted data from, so `/admin/reshard:apply`/`:evict`'s
/// mutations — which bypass `WriteCoordinator`/the AOF — reach durability
/// before the driver triggers the cutover rolling restart, instead of
/// depending on the next periodic `LUMEN_SNAPSHOT_SECS` tick. `persisted:
/// false` means no durable store is configured on this node (nothing to
/// lose on restart, e.g. dev mode); a production/operator deployment with
/// segment persistence configured always reports `true` on success. See
/// [`CheckpointSink`].
#[utoipa::path(
    post,
    path = "/admin/checkpoint",
    tag = "Admin",
    responses(
        (status = 200, description = "Checkpoint committed (or vacuously satisfied if no durable store is configured)", body = serde_json::Value),
        (status = 400, description = "Checkpoint write failed", body = ApiError)
    )
)]
async fn admin_checkpoint(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    auth.ensure("*", Role::Admin)?;
    let persisted = state
        .checkpoint
        .checkpoint_now()
        .await
        .map_err(ApiErr::from)?;
    tracing::info!(
        target: "lumen.audit",
        event = "admin_checkpoint",
        subject = auth.subject().unwrap_or("anonymous"),
        persisted,
    );
    Ok(Json(serde_json::json!({ "persisted": persisted })))
}

fn default_fence_ttl_secs() -> u64 {
    300
}

/// Upper bound on `ReshardFenceRequest::ttl_secs` (#1443 R3): well above any
/// real driver tick, but small enough that `Instant::now().checked_add` never
/// overflows and a malformed/malicious admin request can never arm an
/// effectively-permanent write pause.
const MAX_FENCE_TTL_SECS: u64 = 3600;

#[derive(Debug, Deserialize)]
struct ReshardFenceRequest {
    /// Same `virtual_bucket_count` the caller's map uses, matching
    /// [`ScopedBackupRequest`]'s convention.
    virtual_bucket_count: u32,
    /// Buckets to pause writes on. An empty set explicitly clears any
    /// currently-armed fence, independent of its deadline.
    buckets: BTreeSet<u32>,
    /// How long the pause stays armed if never explicitly cleared (a
    /// crashed-driver backstop; see [`WriteFence`]'s doc). Defaults to 300s —
    /// generous relative to one driver tick (`DRIVER_POLL_INTERVAL`, 20s in
    /// `reshard_driver.rs`) plus a full migration-pass HTTP round trip, while
    /// still bounded well under any operator-visible SLO.
    #[serde(default = "default_fence_ttl_secs")]
    ttl_secs: u64,
}

/// `POST /admin/reshard:fence`: arm or clear a bounded write pause on a set
/// of virtual buckets (#1396 R2). The reshard driver's cutover
/// (`operator::reshard_driver::advance_catching_up`) arms this over exactly
/// the buckets its final `CatchingUp` migration pass is about to copy,
/// immediately before that pass, and clears it (`buckets: []`) once the
/// pass/evict/checkpoint/cutover sequence finishes — on success or on
/// `Blocked`. A write to a fenced bucket is rejected with `503
/// bucket_write_paused` rather than silently dropped or applied against a
/// map that is about to change; see [`WriteFence`] for the crash-safety
/// (TTL) argument.
#[utoipa::path(
    post,
    path = "/admin/reshard:fence",
    tag = "Admin",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Fence armed or cleared", body = serde_json::Value),
        (status = 400, description = "Invalid virtual_bucket_count", body = ApiError)
    )
)]
async fn reshard_fence(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ReshardFenceRequest>,
) -> Result<Json<serde_json::Value>, ApiErr> {
    auth.ensure("*", Role::Admin)?;
    if req.virtual_bucket_count == 0 {
        return Err(ApiErr::new(
            StatusCode::BAD_REQUEST,
            "invalid_virtual_bucket_count",
            "virtual_bucket_count must be > 0",
        ));
    }
    if req.buckets.is_empty() {
        state.write_fence.clear();
        tracing::info!(
            target: "lumen.audit",
            event = "reshard_fence_cleared",
            subject = auth.subject().unwrap_or("anonymous"),
        );
    } else {
        // #1443 R3: reject a nonsensical TTL as 400 rather than letting
        // `WriteFence::arm`'s `Instant::now() + ttl` overflow — `0` would
        // arm-then-immediately-expire (never actually pausing anything, a
        // silent no-op the caller would wrongly believe closed the write
        // window), and anything above the generous upper bound is either a
        // malformed request or would arm an effectively-permanent pause.
        if req.ttl_secs == 0 || req.ttl_secs > MAX_FENCE_TTL_SECS {
            return Err(ApiErr::new(
                StatusCode::BAD_REQUEST,
                "invalid_ttl_secs",
                format!(
                    "ttl_secs must be in 1..={MAX_FENCE_TTL_SECS}, got {}",
                    req.ttl_secs
                ),
            ));
        }
        if !state.write_fence.arm(
            req.virtual_bucket_count,
            req.buckets.clone(),
            Duration::from_secs(req.ttl_secs),
        ) {
            // Unreachable in practice now that ttl_secs is bounded above,
            // but `arm` still reports overflow explicitly (#1443 R3) rather
            // than panicking — surface it as the same 400 shape instead of a
            // silently-unarmed 200.
            return Err(ApiErr::new(
                StatusCode::BAD_REQUEST,
                "invalid_ttl_secs",
                "ttl_secs would overflow the fence deadline",
            ));
        }
        tracing::info!(
            target: "lumen.audit",
            event = "reshard_fence_armed",
            subject = auth.subject().unwrap_or("anonymous"),
            virtual_bucket_count = req.virtual_bucket_count,
            buckets = req.buckets.len(),
            ttl_secs = req.ttl_secs,
        );
    }
    Ok(Json(serde_json::json!({
        "armed": !req.buckets.is_empty(),
        "buckets": req.buckets,
    })))
}

// ---------------------------------------------------------------------------
// OpenAPI
// ---------------------------------------------------------------------------

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut doc = ApiDoc::openapi();
    doc.info.version = env!("CARGO_PKG_VERSION").to_string();
    inject_query_twins(&mut doc);
    doc
}

/// Describe the #1297 `QUERY` twins (OpenAPI 3.2 / RFC 10008, epic #1296 R1)
/// in the generated document: `QUERY /collections` (twin of `POST
/// /collections:search`) and `QUERY /collections/{collection_id}` (twin of
/// `POST /collections/{collection_id}/search`).
///
/// utoipa 4.2.3 predates OpenAPI 3.2 and has no `PathItemType::Query`
/// variant, so the operation is injected as raw JSON via
/// `PathItem::extensions` — utoipa `#[serde(flatten)]`s that map into the
/// serialized path-item object next to `get`/`post`/etc, giving a `"query"`
/// key byte-identical in shape to a native one. `libs/openapi-codegen`'s IR
/// (`ir/operations.rs`, #1298) only needs that serialized `"query"` key plus
/// an `x-post-twin` extension pointing at the POST twin path; it does not
/// require a typed enum variant to parse the operation. (The `"openapi"`
/// version field itself stays at utoipa's fixed `3.0.3` here — that enum has
/// no 3.2 variant — `lumen spec`'s offline output stamps 3.2 on top; see
/// `spec::openapi_value`.)
fn inject_query_twins(doc: &mut utoipa::openapi::OpenApi) {
    let twin = |doc: &utoipa::openapi::OpenApi, twin_path: &str, operation_id: &str| {
        let mut op = doc
            .paths
            .paths
            .get(twin_path)?
            .operations
            .get(&openapi::PathItemType::Post)?
            .clone();
        op.operation_id = Some(operation_id.to_string());
        op.extensions
            .get_or_insert_with(Default::default)
            .insert("x-post-twin".to_string(), serde_json::json!(twin_path));
        Some(serde_json::to_value(&op).expect("Operation serializes to JSON"))
    };

    if let Some(query_op) = twin(
        doc,
        "/collections/{collection_id}/search",
        "query_collection",
    ) {
        if let Some(item) = doc.paths.paths.get_mut("/collections/{collection_id}") {
            item.extensions
                .get_or_insert_with(Default::default)
                .insert("query".to_string(), query_op);
        }
    }

    if let Some(query_op) = twin(doc, "/collections:search", "query_collections") {
        if let Some(item) = doc.paths.paths.get_mut("/collections") {
            item.extensions
                .get_or_insert_with(Default::default)
                .insert("query".to_string(), query_op);
        }
    }
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

/// HTTP-friendly wrapper that classifies storage errors to status codes.
/// A newtype over the shared `service_http::ApiErr` (status + kind +
/// message, `IntoResponse` renders `service_http::ErrorEnvelope` JSON) —
/// this file keeps only the `StorageError` / `AuthErr` → (status, kind)
/// classification arms. (`crate::types::ApiError` stays a distinct local
/// struct of the same `{error, message}` shape purely for OpenAPI schema
/// identity — see its doc comment.)
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub struct ApiErr(service_http::ApiErr);

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl ApiErr {
    fn new(status: StatusCode, kind: &'static str, message: impl Into<String>) -> Self {
        Self(service_http::ApiErr::new(status, kind, message))
    }

    fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", msg)
    }
}

/// One-hop shard-forward failure — the owning shard was unreachable (pod
/// down/rolling) or its response could not be decoded. Raised via `anyhow`
/// by `routing_remote::RoutedRouter` so `ApiErr`'s classification stays
/// centralized here rather than duplicated in the `operator`-gated module;
/// R2 requires this to surface as a clear, distinctly-kinded retryable
/// error, never a silent local answer.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
#[derive(Debug)]
pub struct ShardForwardUnavailable(pub String);

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl std::fmt::Display for ShardForwardUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl std::error::Error for ShardForwardUnavailable {}

/// The owning shard was reached and answered, but with a non-2xx status
/// (e.g. a forwarded write hit `404`/`422`). Re-emitted locally with the
/// same status so a forwarded error is as legible as a local one; `message`
/// carries the remote's own `{error, message}` envelope verbatim.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
#[derive(Debug)]
pub struct ShardForwardRemoteError {
    pub status: u16,
    pub message: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl std::fmt::Display for ShardForwardRemoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "shard forward error ({}): {}", self.status, self.message)
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl std::error::Error for ShardForwardRemoteError {}

/// A forwarded request declared a shard-map version that disagrees with
/// this pod's own live map (#1442 R2). A rolling restart after a completed
/// reshard split can run pods on two different `SHARD_MAP_*` env snapshots
/// for a bounded window (pods only read env at boot) — rather than let the
/// one-hop guard force a local answer that may be wrong on either side of
/// the split, the receiver rejects with this distinct, retryable error so
/// the caller (or its own retry policy) waits for the rollout to converge.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
#[derive(Debug)]
pub struct ShardMapVersionMismatch {
    pub sender_version: u64,
    pub local_version: u64,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl std::fmt::Display for ShardMapVersionMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "forwarded request's shard-map version {} disagrees with this pod's live version {}",
            self.sender_version, self.local_version
        )
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl std::error::Error for ShardMapVersionMismatch {}

/// A forwarded request's one-hop marker (`x-lumen-forwarded`) claimed this
/// pod, but recomputing ownership from this pod's own shard map disagrees
/// (#1442 R1). The marker alone is caller-controlled — an external client
/// can set it directly on a request to any pod, forcing local handling on a
/// bucket that pod doesn't actually own — so it is now validated on
/// receipt rather than trusted blindly; a spoofed or genuinely misrouted
/// forward is rejected, never honored.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
#[derive(Debug)]
pub struct ShardForwardMisrouted {
    pub bucket: u32,
    pub owner_shard: u32,
    pub local_shard: u32,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl std::fmt::Display for ShardForwardMisrouted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "forwarded request targets virtual bucket {} (owned by shard {}), but this pod is \
             shard {}; refusing to honor an unverified forwarded-hop marker",
            self.bucket, self.owner_shard, self.local_shard
        )
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl std::error::Error for ShardForwardMisrouted {}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl From<anyhow::Error> for ApiErr {
    fn from(e: anyhow::Error) -> Self {
        // #1486 R2: a write waiter released without a genuine apply outcome
        // (dedup-guard skip or a bounded submit() timeout) is transient —
        // surface it as a retryable 503, never the generic 400 fallback
        // below (which would misreport it as a bad request) and never a
        // silent hang (the original defect).
        if e.downcast_ref::<SubmitStalled>().is_some() {
            return Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "write_stalled",
                e.to_string(),
            );
        }
        if e.downcast_ref::<ShardForwardUnavailable>().is_some() {
            return Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "shard_forward_unavailable",
                e.to_string(),
            );
        }
        if let Some(re) = e.downcast_ref::<ShardForwardRemoteError>() {
            let status = StatusCode::from_u16(re.status).unwrap_or(StatusCode::BAD_GATEWAY);
            return Self::new(status, "shard_forwarded_error", re.message.clone());
        }
        if e.downcast_ref::<ShardMapVersionMismatch>().is_some() {
            return Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "shard_map_version_mismatch",
                e.to_string(),
            );
        }
        if e.downcast_ref::<ShardForwardMisrouted>().is_some() {
            return Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "shard_forward_misrouted",
                e.to_string(),
            );
        }
        if let Some(se) = e.downcast_ref::<StorageError>() {
            return match se {
                StorageError::CollectionNotFound(_) => {
                    Self::new(StatusCode::NOT_FOUND, "not_found", e.to_string())
                }
                StorageError::InvalidCollectionName(_) => Self::new(
                    StatusCode::BAD_REQUEST,
                    "invalid_collection_name",
                    e.to_string(),
                ),
                StorageError::UnknownField { .. } => Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "unknown_field",
                    e.to_string(),
                ),
                StorageError::TypeMismatch { .. } => Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "type_mismatch",
                    e.to_string(),
                ),
                StorageError::DuplicatesOnText(_) => {
                    Self::new(StatusCode::BAD_REQUEST, "bad_request", e.to_string())
                }
                StorageError::InvalidNumber(_) => Self::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_number",
                    e.to_string(),
                ),
                StorageError::BulkLimit { .. } => {
                    Self::new(StatusCode::PAYLOAD_TOO_LARGE, "bulk_limit", e.to_string())
                }
                StorageError::QueryTooComplex(_) => {
                    Self::new(StatusCode::BAD_REQUEST, "query_too_complex", e.to_string())
                }
                StorageError::Gone(_) => Self::new(StatusCode::GONE, "gone", e.to_string()),
                StorageError::UnsupportedSort(_) => {
                    Self::new(StatusCode::BAD_REQUEST, "unsupported_sort", e.to_string())
                }
                // #1467 R4: caller-declared `total_chunks` failed the sanity
                // cap — a client/protocol error, not a transient one.
                StorageError::InvalidPruneChunk { .. } => Self::new(
                    StatusCode::BAD_REQUEST,
                    "invalid_prune_chunk",
                    e.to_string(),
                ),
                // #1467 R4: the prune accumulator is at its entry cap — a
                // 429-class signal (retryable once the driver's other,
                // presumably-stuck passes GC out or complete) rather than a
                // permanent 4xx.
                StorageError::PruneAccumulatorFull { .. } => Self::new(
                    StatusCode::TOO_MANY_REQUESTS,
                    "prune_accumulator_full",
                    e.to_string(),
                ),
            };
        }
        Self::new(StatusCode::BAD_REQUEST, "bad_request", e.to_string())
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl IntoResponse for ApiErr {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl From<crate::auth::AuthErr> for ApiErr {
    fn from(e: crate::auth::AuthErr) -> Self {
        match e {
            crate::auth::AuthErr::Forbidden {
                subject,
                needed,
                collection_id,
            } => Self::new(
                StatusCode::FORBIDDEN,
                "forbidden",
                format!("subject `{subject}` lacks {needed:?} on `{collection_id}`"),
            ),
        }
    }
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/api.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/api.rs` captured during lumen
      standardization onto the per-file codegen ladder.
  - path: projects/lumen/src/api.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1396 R2: add a bounded write-pause seam for the reshard driver's
      final `CatchingUp` pass. New `WriteFence` type (armed/cleared via a
      new `POST /admin/reshard:fence` admin route, `ReshardFenceRequest`)
      tracks a set of paused virtual buckets behind a TTL deadline enforced
      by this serving pod itself, independent of the driver process's
      liveness — a crashed driver that never explicitly clears the fence
      cannot wedge writes forever. A new `enforce_write_fence` helper is
      checked in `index`, `replace_docs`, and `replace_doc` (not
      `delete_external_id`, which is safe to race either way); a write to a
      fenced bucket is rejected `503 bucket_write_paused`. `AppState` gained
      a `write_fence: WriteFence` field (defaulted in `with_components`).
  - path: projects/lumen/src/api.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1398 R1-R3: cross-pod shard routing for operator/k8s serving pods.
      New `RoutedBackend` trait (`search`/`index`/`replace_docs`/`delete`,
      all take the inbound `headers` so the one implementation can forward
      `Authorization` + `x-read-consistency`); new `AppState.routed:
      Option<Arc<dyn RoutedBackend>>` field (defaulted `None` in
      `with_components`, wired via new `with_routed`). `search_core`,
      `batch_search_core`, `index`, `delete_external_id`, `replace_docs`,
      and `replace_doc` all branch on `state.routed` first and fall back to
      the existing local `search_backend`/`write_backend` path unchanged
      when it is absent, so `shardCount:1` serving never constructs a
      router (AC5). New `ShardForwardUnavailable`/`ShardForwardRemoteError`
      error types classify forwarding failures distinctly from local
      errors via new `From<anyhow::Error> for ApiErr` downcast arms. The
      `duplicates` handler is deliberately not wired to `state.routed`
      (per-shard `min_group_size` filtering happens before a cross-shard
      merge could be correct) — out of #1398's scope, left local-only.
  - path: projects/lumen/src/api.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1442 round-2 hardening: new `ShardMapVersionMismatch` (R2, `503
      shard_map_version_mismatch`) and `ShardForwardMisrouted` (R1, `503
      shard_forward_misrouted`) error types plus matching
      `From<anyhow::Error> for ApiErr` downcast arms, produced by
      `routing_remote::RoutedRouter`'s new validate-on-receipt checks. `R6`:
      `duplicates` now rejects with `501 duplicates_not_routed` when
      `state.routed.is_some()` instead of silently answering from
      local-shard data only; `reindex_stream` now rejects with `501
      reindex_stream_not_routed` in routed mode for the same reason (its
      spawned batch loop bypasses both per-item shard ownership and
      `enforce_write_fence`), directing callers to `POST .../index` instead.
  - path: projects/lumen/src/api.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1443 R3/R4 hardening of the #1396/#1442 write fence. `WriteFence::arm`
      now computes its deadline with `Instant::checked_add` and returns
      `bool` instead of panicking on overflow; its internal mutex lock is
      poison-proof (`unwrap_or_else(poisoned -> into_inner)`), matching
      `segment_rdb`'s save-lock pattern, so a prior panic can never wedge the
      fence into a permanent outage. `reshard_fence` validates
      `ttl_secs` against a new `MAX_FENCE_TTL_SECS = 3600` bound (`0` or
      `> 3600` is rejected `400 invalid_ttl_secs` before ever reaching
      `arm`) and surfaces `arm`'s new `false` return as the same 400 shape
      instead of a silently-unarmed 200. `reshard_apply` now accepts an
      optional `replace_ids`/`bucket`/`virtual_bucket_count` on
      `ReshardBatch` (R2: authoritative-subset replace for a moving
      bucket's final fenced re-sync, closing a delete-resurrection gap —
      `delete_external_id` itself stays fence-exempt and unchanged) and
      passes the resulting `ReshardBatchReplaceScope` through to
      `Engine::apply_reshard_batch`, logging/returning the new
      `documents_pruned` count alongside `documents_upserted`.
      `enforce_write_fence`'s doc comment now explains the R2 design choice
      and its one disclosed residual gap (see `reshard.rs`'s changelog
      entry for the full rationale).
  - path: projects/lumen/src/api.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1444 R2: `router`'s `DefaultBodyLimit::max(..)` now reads
      `crate::reshard::ADMIN_ROUTE_BODY_LIMIT_BYTES` instead of a
      hand-copied `8 * 1024 * 1024` literal, so this route's actual
      enforced body limit can never drift from the constant the reshard
      driver's oversize-batch detection compares against.
  - path: projects/lumen/src/api.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1458 R2: `delete_external_id` now calls `enforce_write_fence` like
      every other write path, replacing the earlier deliberate DELETE
      exemption — a delete racing strictly inside the sub-window between a
      final fenced pass's scoped-backup read and its eviction is now
      rejected with a retryable 503 `bucket_write_paused` instead of being
      silently accepted. `enforce_write_fence`'s doc comment updated to
      match.
  - path: projects/lumen/src/api.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1467 R4: error mapping for the two new `apply_reshard_prune_chunk`
      accumulator-hardening errors. `StorageError::InvalidPruneChunk`
      (caller-declared `total_chunks` of `0` or beyond the sanity cap) maps
      to `400 invalid_prune_chunk` -- a client/protocol error, not
      transient. `StorageError::PruneAccumulatorFull` (the receiver's
      bounded in-flight-group accumulator is at capacity) maps to
      `429 prune_accumulator_full` -- a retryable signal once the driver's
      other stuck/abandoned passes age-GC out or complete, distinct from a
      permanent 4xx. `batch_search_storage_error`'s reason-string mapping
      updated to match for parity with the single-search error path.
```
