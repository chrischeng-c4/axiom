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
| `ApiDoc` | projects/lumen/src/api.rs | struct | pub | 332 |  |
| `ApiErr` | projects/lumen/src/api.rs | struct | pub | 1081 |  |
| `AppState` | projects/lumen/src/api.rs | struct | pub | 48 |  |
| `new` | projects/lumen/src/api.rs | function | pub | 228 | new(engine: Arc<Engine>, auth: Arc<AuthConfig>) -> Self |
| `open` | projects/lumen/src/api.rs | function | pub | 249 | open(engine: Arc<Engine>) -> Self |
| `openapi` | projects/lumen/src/api.rs | function | pub | 1035 | openapi() -> utoipa::openapi::OpenApi |
| `router` | projects/lumen/src/api.rs | function | pub | 335 | router(state: AppState) -> Router |
| `with_cluster` | projects/lumen/src/api.rs | function | pub | 232 | with_cluster(mut self, cluster: Arc<crate::raft::ClusterState>) -> Self |
| `with_components` | projects/lumen/src/api.rs | function | pub | 207 | with_components(         engine: Arc<Engine>,         auth: Arc<AuthConfig>,         writer: Arc<WriteCoordinator>,     ) -> Self |
| `with_search_backend` | projects/lumen/src/api.rs | function | pub | 237 | with_search_backend(mut self, search_backend: Arc<dyn SearchBackend>) -> Self |
| `with_wal` | projects/lumen/src/api.rs | function | pub | 199 | with_wal(engine: Arc<Engine>, auth: Arc<AuthConfig>, wal: SharedWal) -> Self |
| `with_write_backend` | projects/lumen/src/api.rs | function | pub | 242 | with_write_backend(mut self, write_backend: Arc<dyn WriteBackend>) -> Self |
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

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::{Html, IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use utoipa::OpenApi;

use axum::http::HeaderMap;

use crate::auth::{auth_middleware, AuthConfig, AuthContext, Role};
use crate::backup_sink::{BackupSink, LocalFsSink};
use crate::coordinator::WriteCoordinator;
use crate::log_entry::RaftLogEntry;
use crate::raft::{ClusterStateView, ReadConsistency};
use crate::storage::{ApplyOutcome, DropOutcome, Engine, SnapshotV1, StorageError};
use crate::types::{
    Analyzer, ApiError, CacheStats, CreateCollectionRequest, CreateCollectionResponse,
    DuplicateGroup, DuplicatesRequest, DuplicatesResponse, FieldSpec, FieldStats, FieldType,
    FieldValue, IndexItem, IndexRequest, IndexResponse, KnnQuery, MatchOp, MatchQuery, QueryNode,
    RangeQuery, SearchHit, SearchRequest, SearchResponse, StatsResponse, StorageStats, TermQuery,
    TermsQuery, VectorBackend, VectorMetric, VectorQuantize, VectorSpec,
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
    /// Writes go through the coordinator: publish to the log, wait for
    /// the local apply loop, return the outcome. Reads use `engine`
    /// directly. See `coordinator` / `wal`.
    pub writer: Arc<WriteCoordinator>,
    /// Write/mutation backend. Defaults to the local coordinator; sharded
    /// serving can replace it with a document-router that fans out writes
    /// across independent shard coordinators.
    pub write_backend: Arc<dyn WriteBackend>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub trait SearchBackend: Send + Sync {
    fn search(&self, collection_id: &str, req: SearchRequest) -> Result<SearchResponse>;
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

    async fn delete(
        &self,
        collection_id: String,
        external_id: String,
        field: Option<String>,
    ) -> Result<()>;

    async fn drop_field(&self, collection_id: String, field_name: String) -> Result<u32>;
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
    writer: Arc<WriteCoordinator>,
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
    /// Build state with an explicit write log (e.g. a broker-backed one
    /// for clustered deployments). Spawns the apply loop.
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
        writer: Arc<WriteCoordinator>,
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
        (url = "http://lumen-svc:8080", description = "in-cluster ClusterIP"),
        (url = "http://localhost:8080", description = "local dev")
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
        search,
        duplicates,
        stats,
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
        SearchRequest,
        QueryNode,
        MatchQuery,
        MatchOp,
        TermQuery,
        TermsQuery,
        RangeQuery,
        KnnQuery,
        crate::types::RrfQuery,
        crate::types::ExistsQuery,
        crate::types::DuplicatedQuery,
        SearchHit,
        SearchResponse,
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
    ))
)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub struct ApiDoc;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub fn router(state: AppState) -> Router {
    // Apply auth middleware only to data-plane routes. Admin/Probe
    // endpoints (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
    // `/docs`) stay open so K8s probes and Prometheus scrape can hit
    // them without a token even when auth is required.
    let auth_state = state.auth.clone();
    let data_plane = Router::new()
        .route("/collections", get(list_collections))
        .route(
            "/collections/{collection_id}",
            put(create_collection).delete(drop_collection),
        )
        .route("/collections/{collection_id}/index", post(index))
        .route(
            "/collections/{collection_id}/index/{external_id}",
            delete(delete_external_id),
        )
        .route("/collections/{collection_id}/search", post(search))
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
        .route("/admin/restore", post(restore))
        .layer(from_fn_with_state(auth_state, auth_middleware))
        // Bound request bodies: a bulk index is ~MBs (the item cap is the real
        // guard); 8MiB is the broker payload budget. Rejects oversized
        // bodies with 413 before they hit a handler.
        .layer(axum::extract::DefaultBodyLimit::max(8 * 1024 * 1024));

    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/version", get(version))
        .route("/metrics", get(metrics))
        .route("/debug/cluster", get(debug_cluster))
        .route("/openapi.json", get(openapi_spec))
        .route("/docs", get(docs_swagger))
        .merge(data_plane)
        // One tracing span per HTTP request — structured request logs always, and
        // the source spans the OTLP layer exports as traces when LUMEN_OTLP_ENDPOINT
        // is set. INFO level so the default `info` EnvFilter keeps it.
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
            ),
        )
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/metrics",
    tag = "Admin",
    responses((status = 200, description = "Prometheus text-format metrics", body = String))
)]
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

// ---------------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/healthz",
    tag = "Admin",
    responses((status = 200, description = "Process is alive", body = String))
)]
async fn healthz() -> &'static str {
    "ok"
}

#[utoipa::path(
    get,
    path = "/version",
    tag = "Admin",
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
    responses(
        (status = 200, description = "Engine ready"),
        (status = 503, description = "Not ready")
    )
)]
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
    Path(collection_id): Path<String>,
    Json(req): Json<IndexRequest>,
) -> Result<Json<IndexResponse>, ApiErr> {
    auth.ensure(&collection_id, Role::Write)?;
    let resp = state
        .write_backend
        .index(collection_id.clone(), req)
        .await
        .map_err(ApiErr::from)?;
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
    Path((collection_id, external_id)): Path<(String, String)>,
    Query(q): Query<DeleteQuery>,
) -> Result<StatusCode, ApiErr> {
    auth.ensure(&collection_id, Role::Write)?;
    state
        .write_backend
        .delete(collection_id.clone(), external_id, q.field)
        .await
        .map_err(ApiErr::from)?;
    Ok(StatusCode::NO_CONTENT)
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
    auth.ensure(&collection_id, Role::Read)?;
    let _consistency = read_consistency_from(&headers);
    // Standalone and explicit-broker builds satisfy this locally. Primary-
    // replica mode will enforce leader/bounded/any against the live cluster
    // state once the raftcore-backed surface is wired.
    Ok(Json(
        state
            .search_backend
            .search(&collection_id, req)
            .map_err(ApiErr::from)?,
    ))
}

#[utoipa::path(
    post,
    path = "/collections/{collection_id}/duplicates",
    tag = "Query",
    params(("collection_id" = String, Path, description = "Collection namespace")),
    request_body = DuplicatesRequest,
    responses((status = 200, description = "Duplicate groups", body = DuplicatesResponse))
)]
async fn duplicates(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Path(collection_id): Path<String>,
    Json(req): Json<DuplicatesRequest>,
) -> Result<Json<DuplicatesResponse>, ApiErr> {
    auth.ensure(&collection_id, Role::Read)?;
    let _consistency = read_consistency_from(&headers);
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
        .map_err(|e| ApiErr {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            kind: "stream_init",
            message: e.to_string(),
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
    let payload = serde_json::to_vec(&snap).map_err(|e| ApiErr {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        kind: "encode",
        message: e.to_string(),
    })?;
    let sink = LocalFsSink::new(&req.path, &req.prefix).map_err(|e| ApiErr {
        status: StatusCode::BAD_REQUEST,
        kind: "bad_sink",
        message: e.to_string(),
    })?;
    let key = sink
        .put(std::time::SystemTime::now(), &payload)
        .map_err(|e| ApiErr {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            kind: "sink_put",
            message: e.to_string(),
        })?;
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
// OpenAPI
// ---------------------------------------------------------------------------

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut doc = ApiDoc::openapi();
    doc.info.version = env!("CARGO_PKG_VERSION").to_string();
    doc
}

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(openapi())
}

/// Interactive Swagger UI at `/docs` (FastAPI convention). The page
/// pulls the live spec from `/openapi.json`, so its "Try it out"
/// buttons fire real requests against this pod — handy for exploring
/// `match` / `term` / `range` / `knn` queries from a browser.
async fn docs_swagger() -> Html<&'static str> {
    Html(
        r##"<!doctype html>
<html>
  <head>
    <title>lumen API</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css" />
    <style>body { margin: 0; }</style>
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
      window.ui = SwaggerUIBundle({
        url: "/openapi.json",
        dom_id: "#swagger-ui",
        deepLinking: true,
      });
    </script>
  </body>
</html>"##,
    )
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

/// HTTP-friendly wrapper that classifies storage errors to status codes.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
pub struct ApiErr {
    status: StatusCode,
    kind: &'static str,
    message: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl ApiErr {
    fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            kind: "not_found",
            message: msg.into(),
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl From<anyhow::Error> for ApiErr {
    fn from(e: anyhow::Error) -> Self {
        if let Some(se) = e.downcast_ref::<StorageError>() {
            return match se {
                StorageError::CollectionNotFound(_) => Self {
                    status: StatusCode::NOT_FOUND,
                    kind: "not_found",
                    message: e.to_string(),
                },
                StorageError::UnknownField { .. } => Self {
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                    kind: "unknown_field",
                    message: e.to_string(),
                },
                StorageError::TypeMismatch { .. } => Self {
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                    kind: "type_mismatch",
                    message: e.to_string(),
                },
                StorageError::DuplicatesOnText(_) => Self {
                    status: StatusCode::BAD_REQUEST,
                    kind: "bad_request",
                    message: e.to_string(),
                },
                StorageError::InvalidNumber(_) => Self {
                    status: StatusCode::UNPROCESSABLE_ENTITY,
                    kind: "invalid_number",
                    message: e.to_string(),
                },
                StorageError::BulkLimit { .. } => Self {
                    status: StatusCode::PAYLOAD_TOO_LARGE,
                    kind: "bulk_limit",
                    message: e.to_string(),
                },
                StorageError::QueryTooComplex(_) => Self {
                    status: StatusCode::BAD_REQUEST,
                    kind: "query_too_complex",
                    message: e.to_string(),
                },
                StorageError::Gone(_) => Self {
                    status: StatusCode::GONE,
                    kind: "gone",
                    message: e.to_string(),
                },
                StorageError::UnsupportedSort(_) => Self {
                    status: StatusCode::BAD_REQUEST,
                    kind: "unsupported_sort",
                    message: e.to_string(),
                },
            };
        }
        Self {
            status: StatusCode::BAD_REQUEST,
            kind: "bad_request",
            message: e.to_string(),
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl IntoResponse for ApiErr {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(ApiError {
                error: self.kind.to_string(),
                message: self.message,
            }),
        )
            .into_response()
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-api-rs.md#source
impl From<crate::auth::AuthErr> for ApiErr {
    fn from(e: crate::auth::AuthErr) -> Self {
        match e {
            crate::auth::AuthErr::Unauthenticated => Self {
                status: StatusCode::UNAUTHORIZED,
                kind: "unauthenticated",
                message: "valid bearer token required".into(),
            },
            crate::auth::AuthErr::Forbidden {
                subject,
                needed,
                collection_id,
            } => Self {
                status: StatusCode::FORBIDDEN,
                kind: "forbidden",
                message: format!("subject `{subject}` lacks {needed:?} on `{collection_id}`"),
            },
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
```
