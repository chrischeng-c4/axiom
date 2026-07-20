//! HTTP/2 (h2c) service layer — the callable vector-database front end.
//!
//! `beam serve` turns the in-process engine (collection store + flat/IVF indexes)
//! into a real REST service. One port speaks HTTP/1.1 and HTTP/2 cleartext (h2c,
//! prior-knowledge) via the shared [`h2c::serve`] transport (the ecosystem's
//! drop-in for `axum::serve`, which is HTTP/1 only), exactly like keep / loom /
//! lumen. A plain HTTP/1.1 REST client works too.
//!
//! ## What it owns
//!
//! An in-process registry ([`Registry`]) maps a collection name to its
//! [`CollectionState`] — the [`Collection`] plus, on a GPU host, a rebuilt
//! [`GpuFlatIndex`]. Query runs on the GPU flat path when a [`GpuContext`] is
//! available and falls back to the exact CPU flat oracle
//! ([`CpuFlatIndex`](crate::index::cpu_flat::CpuFlatIndex)) otherwise — the same
//! graceful GPU-or-CPU choice the tests and bench make. The index is rebuilt on
//! every mutation (correctness over peak throughput).
//!
//! ## Routes
//!
//! `GET /healthz` and `GET /readyz` are the k8s probes. The data plane is under
//! `/v1/collections`: create / list / drop a collection, batch-upsert or delete
//! vectors, and run a (optionally filtered) k-NN query. Request and response
//! bodies are JSON; the payload and filter JSON shapes map onto the engine
//! [`Payload`] / [`Filter`] types.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use anyhow::Context;

use axum::extract::{Path, State, Extension};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;
use utoipa::{OpenApi, ToSchema};

use crate::collection::{Collection, Metric};
use crate::gpu::{GpuContext, GpuFlatIndex};
use crate::index::cpu_flat::CpuFlatIndex;
use crate::index::{Neighbor, VectorIndex};
use crate::payload::{AttrValue, Clause, Filter, Payload};

use service_http::ApiErr;

/// The in-process collection registry: name to its live [`CollectionState`],
/// behind a single reader-writer lock (queries take a read lock; mutations a
/// write lock).
type Registry = Arc<RwLock<HashMap<String, CollectionState>>>;

/// One registered collection: the engine [`Collection`] plus, on a GPU host, a
/// [`GpuFlatIndex`] rebuilt from it on every mutation. `gpu_index` is `None` on a
/// GPU-less host or while the collection is physically empty; query then uses the
/// CPU flat oracle.
pub struct CollectionState {
    pub collection: Collection,
    pub gpu_index: Option<GpuFlatIndex>,
}

impl CollectionState {
    pub fn new(collection: Collection) -> Self {
        Self {
            collection,
            gpu_index: None,
        }
    }

    /// Rebuild the GPU flat index from the current collection when a GPU is
    /// present and the collection holds physical rows (a GPU buffer must be
    /// non-empty). Called after every mutation so the index reflects the store.
    pub fn rebuild(&mut self, gpu: &Option<Arc<GpuContext>>) {
        self.gpu_index = match gpu {
            Some(ctx) if self.collection.capacity() > 0 => {
                Some(GpuFlatIndex::new(ctx, &self.collection))
            }
            _ => None,
        };
    }

    /// Run a k-NN query, filtered when `filter` has clauses. Uses the GPU flat
    /// index when built, else the exact CPU flat oracle over the collection.
    fn query(&self, query: &[f32], k: usize, filter: &Filter) -> Vec<Neighbor> {
        match &self.gpu_index {
            Some(idx) if filter.is_empty() => idx.search_knn(query, k),
            Some(idx) => idx.search_knn_filtered(query, k, filter),
            None if filter.is_empty() => CpuFlatIndex::new(&self.collection).search_knn(query, k),
            None => CpuFlatIndex::new(&self.collection).search_knn_filtered(query, k, filter),
        }
    }
}

// <HANDWRITE gap="missing-generator:logic--7bf4e2c0" tracker="#2149" reason="logic section in service.rs is hand-written pending codegen support">
/// Shared handler state: the registry plus the optional GPU context used to
/// (re)build indexes. Cheap to clone (both fields are `Arc`-backed).
#[derive(Clone)]
pub struct AppState {
    pub registry: Registry,
    pub gpu: Option<Arc<GpuContext>>,
    pub data_dir: Option<std::path::PathBuf>,
    pub auth: Arc<StaticRoleMapVerifier>,
}

fn persist_collection(state: &AppState, name: &str, col: &Collection) {
    if let Some(ref dir) = state.data_dir {
        let path = dir.join(format!("{}.bin", name));
        if let Err(e) = col.save(&path) {
            eprintln!("Failed to persist collection `{}`: {e}", name);
        }
    }
}

fn delete_collection_file(state: &AppState, name: &str) {
    if let Some(ref dir) = state.data_dir {
        let path = dir.join(format!("{}.bin", name));
        let _ = std::fs::remove_file(&path);
    }
}

// --- Auth Config & Helpers ---
pub const AUTH_MODE_ENV: &str = "BEAM_AUTH";
pub const TOKEN_REGISTRY_FILE_ENV: &str = "BEAM_TOKEN_REGISTRY_FILE";
pub const LEGACY_TOKENS_ENV: &str = "BEAM_TOKENS";

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
    ) -> anyhow::Result<Self> {
        let required = match mode.trim().to_ascii_lowercase().as_str() {
            "required" => true,
            "" | "off" | "disabled" => false,
            other => anyhow::bail!(
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

    pub fn verifier(&self) -> StaticRoleMapVerifier {
        StaticRoleMapVerifier::new(self.required, self.tokens.clone())
    }
}

use service_auth::{Role, RoleMapPrincipal, StaticRoleMapVerifier, TokenClaims};

#[allow(dead_code)]
fn unauthorized_err(message: impl Into<String>) -> ApiErr {
    ApiErr::new(StatusCode::UNAUTHORIZED, "unauthorized", message)
}

fn forbidden_err(message: impl Into<String>) -> ApiErr {
    ApiErr::new(StatusCode::FORBIDDEN, "forbidden", message)
}

fn authorize(
    principal: &RoleMapPrincipal,
    resource: &str,
    needed: Role,
) -> Result<(), ApiErr> {
    principal.ensure(resource, needed).map_err(|denied| {
        forbidden_err(format!(
            "subject `{}` lacks {:?} on `{}`",
            denied.subject, denied.needed, denied.resource
        ))
    })
}
// </HANDWRITE>

// -- Wire types ------------------------------------------------------------

/// `POST /v1/collections` body.
#[derive(Deserialize, ToSchema)]
pub struct CreateCollectionReq {
    name: String,
    dim: usize,
    metric: String,
}

/// One entry in the `GET /v1/collections` list: the name and its LIVE vector count.
#[derive(Serialize, ToSchema)]
pub struct CollectionInfo {
    name: String,
    size: usize,
}

/// `POST /v1/collections/{name}/vectors` body: a batch of upserts.
#[derive(Deserialize, ToSchema)]
pub struct UpsertReq {
    items: Vec<VectorItem>,
}

/// One vector to upsert: external id, the raw `dim`-long vector, and an optional
/// flat attribute payload (`{key: int-or-string}`).
#[derive(Deserialize, ToSchema)]
pub struct VectorItem {
    id: String,
    vector: Vec<f32>,
    #[serde(default)]
    payload: Option<HashMap<String, serde_json::Value>>,
}

/// `POST /v1/collections/{name}/query` body. `nprobe` is accepted for API
/// compatibility with the IVF backends but ignored by the flat query path.
#[derive(Deserialize, ToSchema)]
pub struct QueryReq {
    vector: Vec<f32>,
    k: usize,
    #[serde(default)]
    nprobe: Option<usize>,
    #[serde(default)]
    filter: Option<WireFilter>,
}

/// JSON filter: a conjunction (AND) of [`WireClause`]s.
#[derive(Deserialize, ToSchema)]
pub struct WireFilter {
    clauses: Vec<WireClause>,
}

/// One JSON filter clause. `op = "eq"` needs `value` (int or string);
/// `op = "range"` needs integer `lo` and `hi` (inclusive).
#[derive(Deserialize, ToSchema)]
pub struct WireClause {
    op: String,
    key: String,
    #[serde(default)]
    value: Option<serde_json::Value>,
    #[serde(default)]
    lo: Option<i64>,
    #[serde(default)]
    hi: Option<i64>,
}

/// `POST /v1/collections/{name}/query` response.
#[derive(Serialize, ToSchema)]
pub struct QueryResp {
    neighbors: Vec<NeighborOut>,
}

/// One returned neighbor: external id, raw metric score, and the row payload
/// (omitted when empty). Payload keys are emitted sorted for deterministic output.
#[derive(Serialize, ToSchema)]
pub struct NeighborOut {
    id: String,
    score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<BTreeMap<String, serde_json::Value>>,
}

// -- Error helpers ---------------------------------------------------------

fn bad_request_err(message: impl Into<String>) -> ApiErr {
    ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", message)
}

fn not_found_err(message: impl Into<String>) -> ApiErr {
    ApiErr::new(StatusCode::NOT_FOUND, "not_found", message)
}

fn conflict_err(message: impl Into<String>) -> ApiErr {
    ApiErr::new(StatusCode::CONFLICT, "conflict", message)
}

fn internal_err(message: impl Into<String>) -> ApiErr {
    ApiErr::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_error",
        message,
    )
}

// -- JSON <-> engine mapping ----------------------------------------------

/// Map a JSON scalar to an engine [`AttrValue`]: an integer becomes
/// [`AttrValue::Int`], a string [`AttrValue::Str`]; anything else is rejected.
fn json_to_attr(v: &serde_json::Value) -> Option<AttrValue> {
    match v {
        serde_json::Value::Number(n) => n.as_i64().map(AttrValue::Int),
        serde_json::Value::String(s) => Some(AttrValue::Str(s.clone())),
        _ => None,
    }
}

/// Map an engine [`AttrValue`] back to its JSON scalar.
fn attr_to_json(a: &AttrValue) -> serde_json::Value {
    match a {
        AttrValue::Int(i) => json!(i),
        AttrValue::Str(s) => json!(s),
    }
}

/// Build an engine [`Payload`] from the optional wire `{key: value}` map,
/// rejecting a non-scalar attribute value with a 400.
fn build_payload(map: Option<HashMap<String, serde_json::Value>>) -> Result<Payload, ApiErr> {
    let mut payload = Payload::new();
    for (key, value) in map.into_iter().flatten() {
        let attr = json_to_attr(&value).ok_or_else(|| {
            bad_request_err(format!(
                "payload attribute `{key}` must be an integer or string"
            ))
        })?;
        payload.insert(key, attr);
    }
    Ok(payload)
}

/// Build a single engine [`Clause`] from a wire clause, validating its operands.
fn build_clause(clause: WireClause) -> Result<Clause, ApiErr> {
    match clause.op.as_str() {
        "eq" => {
            let value = clause
                .value
                .ok_or_else(|| bad_request_err("`eq` clause requires `value`"))?;
            let attr = json_to_attr(&value).ok_or_else(|| {
                bad_request_err("`eq` clause `value` must be an integer or string")
            })?;
            Ok(Clause::Eq(clause.key, attr))
        }
        "range" => {
            let lo = clause
                .lo
                .ok_or_else(|| bad_request_err("`range` clause requires integer `lo`"))?;
            let hi = clause
                .hi
                .ok_or_else(|| bad_request_err("`range` clause requires integer `hi`"))?;
            Ok(Clause::IntRange(clause.key, lo, hi))
        }
        other => Err(bad_request_err(format!(
            "unknown clause op `{other}` (expected `eq` or `range`)"
        ))),
    }
}

/// Build an engine [`Filter`] (AND of clauses) from the optional wire filter.
fn build_filter(filter: Option<WireFilter>) -> Result<Filter, ApiErr> {
    let mut out = Filter::new();
    if let Some(filter) = filter {
        for clause in filter.clauses {
            out = out.and(build_clause(clause)?);
        }
    }
    Ok(out)
}

/// Render a row payload as a sorted JSON `{key: scalar}` map (deterministic order).
fn payload_to_map(payload: &Payload) -> BTreeMap<String, serde_json::Value> {
    payload
        .tags
        .iter()
        .map(|(k, v)| (k.clone(), attr_to_json(v)))
        .collect()
}

// -- Handlers --------------------------------------------------------------

/// `POST /v1/collections` — create a collection; 409 if the name is taken, 400
/// on an unknown metric or zero dimension.
#[utoipa::path(
    post,
    path = "/v1/collections",
    request_body = CreateCollectionReq,
    responses(
        (status = 201, description = "Collection created successfully"),
        (status = 400, description = "Bad request", body = ErrorEnvelope),
        (status = 409, description = "Conflict", body = ErrorEnvelope)
    )
)]
async fn create_collection(
    State(state): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Json(req): Json<CreateCollectionReq>,
) -> Result<Response, ApiErr> {
    authorize(&principal, &req.name, Role::Write)?;
    let metric = Metric::parse(&req.metric).ok_or_else(|| {
        bad_request_err(format!(
            "unknown metric `{}` (expected l2|dot|cosine)",
            req.metric
        ))
    })?;
    if req.dim == 0 {
        return Err(bad_request_err("dim must be greater than 0"));
    }

    let mut registry = state.registry.write().expect("registry lock poisoned");
    if registry.contains_key(&req.name) {
        return Err(conflict_err(format!(
            "collection `{}` already exists",
            req.name
        )));
    }
    let mut cs = CollectionState::new(Collection::new(req.name.clone(), req.dim, metric));
    cs.rebuild(&state.gpu);
    // <HANDWRITE gap="missing-generator:logic--7bf4e2c0" tracker="#2149" reason="persist collection">
persist_collection(&state, &req.name, &cs.collection);
    // </HANDWRITE>
    registry.insert(req.name.clone(), cs);

    Ok((
        StatusCode::CREATED,
        Json(json!({ "name": req.name, "dim": req.dim, "metric": req.metric })),
    )
        .into_response())
}

/// `GET /v1/collections` — list every collection name and its live vector count,
/// sorted by name for a deterministic response.
#[utoipa::path(
    get,
    path = "/v1/collections",
    responses(
        (status = 200, description = "Collections listed successfully", body = inline(Vec<CollectionInfo>))
    )
)]
async fn list_collections(
    State(state): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
) -> Response {
    if let Err(deny) = authorize(&principal, "*", Role::Read) {
        return deny.into_response();
    }
    let registry = state.registry.read().expect("registry lock poisoned");
    let mut collections: Vec<CollectionInfo> = registry
        .iter()
        .map(|(name, cs)| CollectionInfo {
            name: name.clone(),
            size: cs.collection.len(),
        })
        .collect();
    collections.sort_by(|a, b| a.name.cmp(&b.name));
    (StatusCode::OK, Json(json!({ "collections": collections }))).into_response()
}

/// `DELETE /v1/collections/{name}` — drop a collection; 404 if it is unknown.
#[utoipa::path(
    delete,
    path = "/v1/collections/{name}",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    responses(
        (status = 200, description = "Collection dropped successfully"),
        (status = 404, description = "Collection not found", body = ErrorEnvelope)
    )
)]
async fn drop_collection(
    State(state): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(name): Path<String>,
) -> Result<Response, ApiErr> {
    authorize(&principal, &name, Role::Write)?;
    let mut registry = state.registry.write().expect("registry lock poisoned");
    if registry.remove(&name).is_some() {
        // <HANDWRITE gap="missing-generator:logic--7bf4e2c0" tracker="#2149" reason="delete collection file">
delete_collection_file(&state, &name);
        // </HANDWRITE>
        Ok((StatusCode::OK, Json(json!({ "dropped": name }))).into_response())
    } else {
        Err(not_found_err(format!("collection `{name}` not found")))
    }
}

/// `POST /v1/collections/{name}/vectors` — batch upsert. The whole batch is
/// dim-validated before any row is applied (a bad item rejects the batch with a
/// 400); 404 if the collection is unknown. Rebuilds the index once afterward.
#[utoipa::path(
    post,
    path = "/v1/collections/{name}/vectors",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    request_body = UpsertReq,
    responses(
        (status = 200, description = "Vectors upserted successfully"),
        (status = 400, description = "Bad request", body = ErrorEnvelope),
        (status = 404, description = "Collection not found", body = ErrorEnvelope)
    )
)]
async fn upsert_vectors(
    State(state): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(name): Path<String>,
    Json(req): Json<UpsertReq>,
) -> Result<Response, ApiErr> {
    authorize(&principal, &name, Role::Write)?;
    let mut registry = state.registry.write().expect("registry lock poisoned");
    let cs = registry
        .get_mut(&name)
        .ok_or_else(|| not_found_err(format!("collection `{name}` not found")))?;
    let dim = cs.collection.dim();

    // Validate every item up front so the batch applies all-or-nothing.
    let mut prepared = Vec::with_capacity(req.items.len());
    for item in req.items {
        if item.vector.len() != dim {
            return Err(bad_request_err(format!(
                "vector for id `{}` has dim {}, expected {}",
                item.id,
                item.vector.len(),
                dim
            )));
        }
        let payload = build_payload(item.payload)?;
        prepared.push((item.id, item.vector, payload));
    }

    let count = prepared.len();
    for (id, vector, payload) in prepared {
        cs.collection
            .upsert(id, &vector, payload)
            .map_err(|e| bad_request_err(e.to_string()))?;
    }
    cs.rebuild(&state.gpu);
    // <HANDWRITE gap="missing-generator:logic--7bf4e2c0" tracker="#2149" reason="persist collection after upsert">
persist_collection(&state, &name, &cs.collection);
    // </HANDWRITE>

    Ok((StatusCode::OK, Json(json!({ "upserted": count }))).into_response())
}

/// `DELETE /v1/collections/{name}/vectors/{id}` — delete one vector; 404 if the
/// collection or the id is unknown. Rebuilds the index afterward.
#[utoipa::path(
    delete,
    path = "/v1/collections/{name}/vectors/{id}",
    params(
        ("name" = String, Path, description = "Collection name"),
        ("id" = String, Path, description = "Vector ID")
    ),
    responses(
        (status = 200, description = "Vector deleted successfully"),
        (status = 404, description = "Collection or vector not found", body = ErrorEnvelope)
    )
)]
async fn delete_vector(
    State(state): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Response, ApiErr> {
    authorize(&principal, &name, Role::Write)?;
    let mut registry = state.registry.write().expect("registry lock poisoned");
    let cs = registry
        .get_mut(&name)
        .ok_or_else(|| not_found_err(format!("collection `{name}` not found")))?;
    if cs.collection.delete(&id) {
        cs.rebuild(&state.gpu);
        // <HANDWRITE gap="missing-generator:logic--7bf4e2c0" tracker="#2149" reason="persist collection after delete">
persist_collection(&state, &name, &cs.collection);
        // </HANDWRITE>
        Ok((StatusCode::OK, Json(json!({ "deleted": id }))).into_response())
    } else {
        Err(not_found_err(format!(
            "vector `{id}` not found in collection `{name}`"
        )))
    }
}

/// `POST /v1/collections/{name}/query` — k-NN query, optionally filtered. The
/// (possibly GPU-blocking) search runs on a blocking task so the async worker
/// stays free. 404 if the collection is unknown, 400 on a dim mismatch or a
/// malformed filter clause.
#[utoipa::path(
    post,
    path = "/v1/collections/{name}/query",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    request_body = QueryReq,
    responses(
        (status = 200, description = "Query executed successfully", body = QueryResp),
        (status = 400, description = "Bad request", body = ErrorEnvelope),
        (status = 404, description = "Collection not found", body = ErrorEnvelope)
    )
)]
async fn query_collection(
    State(state): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(name): Path<String>,
    Json(req): Json<QueryReq>,
) -> Result<Response, ApiErr> {
    authorize(&principal, &name, Role::Read)?;
    let QueryReq {
        vector,
        k,
        nprobe,
        filter,
    } = req;
    // The flat query path is exact/brute-force; nprobe (an IVF knob) is ignored.
    let _ = nprobe;
    let filter = build_filter(filter)?;

    let neighbors = tokio::task::spawn_blocking(move || -> Result<Vec<NeighborOut>, ApiErr> {
        let registry = state.registry.read().expect("registry lock poisoned");
        let cs = registry
            .get(&name)
            .ok_or_else(|| not_found_err(format!("collection `{name}` not found")))?;
        if vector.len() != cs.collection.dim() {
            return Err(bad_request_err(format!(
                "query vector has dim {}, expected {}",
                vector.len(),
                cs.collection.dim()
            )));
        }
        let hits = cs.query(&vector, k, &filter);
        Ok(hits
            .into_iter()
            .map(|n| {
                let payload = cs.collection.payload(n.row as usize);
                let payload = (!payload.is_empty()).then(|| payload_to_map(payload));
                NeighborOut {
                    id: n.external_id,
                    score: n.score,
                    payload,
                }
            })
            .collect())
    })
    .await
    .map_err(|e| internal_err(format!("query task failed: {e}")))??;

    Ok((StatusCode::OK, Json(QueryResp { neighbors })).into_response())
}

// -- Admin Handlers --------------------------------------------------------

/// `GET /admin/backup` — snapshot of the entire database state as CBOR + lz4.
#[utoipa::path(
    get,
    path = "/admin/backup",
    responses(
        (status = 200, description = "Snapshot returned successfully", body = inline(Vec<u8>)),
        (status = 500, description = "Internal error", body = ErrorEnvelope)
    )
)]
async fn admin_backup(
    State(state): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
) -> Result<Response, ApiErr> {
    authorize(&principal, "*", Role::Admin)?;
    let registry = state.registry.read().expect("registry lock poisoned");
    let mut collections = HashMap::new();
    for (name, cs) in registry.iter() {
        collections.insert(name.clone(), cs.collection.clone());
    }
    let snap = crate::persist::BeamSnapshot { collections };
    let bytes = snap.encode().map_err(|e| internal_err(e.to_string()))?;
    Ok((
        StatusCode::OK,
        [("content-type", "application/octet-stream")],
        bytes,
    )
        .into_response())
}

/// `POST /admin/restore` — restore the database from a backup snapshot.
#[utoipa::path(
    post,
    path = "/admin/restore",
    responses(
        (status = 200, description = "Database state restored successfully"),
        (status = 400, description = "Bad request", body = ErrorEnvelope),
        (status = 500, description = "Internal error", body = ErrorEnvelope)
    )
)]
async fn admin_restore(
    State(state): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    body: axum::body::Bytes,
) -> Result<Response, ApiErr> {
    authorize(&principal, "*", Role::Admin)?;
    let snap = crate::persist::BeamSnapshot::decode(&body)
        .map_err(|e| bad_request_err(format!("invalid snapshot body: {e}")))?;

    let mut registry = state.registry.write().expect("registry lock poisoned");
    registry.clear();
    // <HANDWRITE gap="missing-generator:logic--7bf4e2c0" tracker="#2149" reason="clean and persist collections on restore">
if let Some(ref dir) = state.data_dir {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("bin") {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
    for (name, col) in snap.collections {
        let mut cs = CollectionState::new(col);
        cs.rebuild(&state.gpu);
        persist_collection(&state, &name, &cs.collection);
        registry.insert(name, cs);
    }
    // </HANDWRITE>

    Ok((StatusCode::OK, Json(json!({ "status": "restored" }))).into_response())
}

// -- OpenAPI definition ----------------------------------------------------

#[derive(OpenApi)]
#[openapi(
    info(
        title = "beam",
        description = "GPU-native vector database. Owns GPU ANN vector storage, index lifecycle, batch ingest, and vector query execution; distinct from lumen (mixed search / ranking / dedup).",
        license(name = "MIT")
    ),
    servers(
        (url = "http://localhost:7373", description = "local dev")
    ),
    tags(
        (name = "Collections", description = "Collection schema lifecycle"),
        (name = "Vectors",     description = "Vector batch upsert & delete"),
        (name = "Query",       description = "k-NN queries"),
        (name = "Admin",       description = "Backup, restore, OpenAPI")
    ),
    paths(
        create_collection,
        list_collections,
        drop_collection,
        upsert_vectors,
        delete_vector,
        query_collection,
        admin_backup,
        admin_restore,
    ),
    components(
        schemas(
            CreateCollectionReq,
            CollectionInfo,
            UpsertReq,
            VectorItem,
            QueryReq,
            WireFilter,
            WireClause,
            QueryResp,
            NeighborOut,
            service_http::ErrorEnvelope,
        )
    )
)]
pub struct BeamApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    BeamApi::openapi()
}

// -- Assembly + serve ------------------------------------------------------

// <HANDWRITE gap="missing-generator:logic--7bf4e2c0" tracker="#2150" reason="router with state and data_dir support">
pub fn router(gpu: Option<Arc<GpuContext>>) -> Router {
    let registry = Arc::new(RwLock::new(HashMap::new()));
    router_with_state(registry, gpu, None, Arc::new(StaticRoleMapVerifier::open()))
}

pub fn router_with_state(
    registry: Registry,
    gpu: Option<Arc<GpuContext>>,
    data_dir: Option<std::path::PathBuf>,
    auth: Arc<StaticRoleMapVerifier>,
) -> Router {
    let state = AppState {
        registry,
        gpu,
        data_dir,
        auth: auth.clone(),
    };
    let data_plane = Router::new()
        .route(
            "/v1/collections",
            post(create_collection).get(list_collections),
        )
        .route("/v1/collections/{name}", delete(drop_collection))
        .route("/v1/collections/{name}/vectors", post(upsert_vectors))
        .route(
            "/v1/collections/{name}/vectors/{id}",
            delete(delete_vector),
        )
        .route("/v1/collections/{name}/query", post(query_collection))
        .route("/admin/backup", get(admin_backup))
        .route("/admin/restore", post(admin_restore))
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
        .route_layer(axum::middleware::from_fn_with_state(
            auth,
            service_auth::auth_middleware::<StaticRoleMapVerifier>,
        ))
        .with_state(state);

    let drain = Arc::new(server_lifecycle::DrainController::new());

    let probes = service_http::standard_probe_routes(
        drain,
        None, // metrics provider
        openapi,
    );

    probes.merge(data_plane).layer(service_http::trace_layer())
}
// </HANDWRITE>

/// Serve `app` on an already-bound `listener` (HTTP/1.1 + h2c on one port) until
/// `shutdown` resolves. Thin delegation to the shared [`service_http::serve`] transport so
/// beam does not hand-roll the hyper-util accept loop. Public so tests can bind an
/// ephemeral listener, read its address, and spawn the server with a pending
/// shutdown.
pub async fn serve_on(
    listener: TcpListener,
    app: Router,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) {
    service_http::serve(listener, app, shutdown).await;
}

// <HANDWRITE gap="missing-generator:logic--7bf4e2c0" tracker="#2150" reason="serve with data_dir recovery support">
/// Run the vector-database service: acquire the GPU (if any), bind `addr`, print
/// the bound address, load registry from data_dir if configured, and serve the REST API until Ctrl-C / SIGTERM.
pub async fn serve(addr: &str, data_dir: Option<std::path::PathBuf>, auth_config: AuthConfig) -> anyhow::Result<()> {
    let gpu = GpuContext::new().map(Arc::new);
    match &gpu {
        Some(ctx) => {
            let (backend, name) = ctx.adapter_info();
            println!("beam: GPU query path enabled ({backend} / {name})");
        }
        None => println!("beam: no GPU adapter — queries use the CPU flat oracle"),
    }

    let mut registry_map = HashMap::new();
    if let Some(ref dir) = data_dir {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("create data dir {}", dir.display()))?;
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("read data dir {}", dir.display()))?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("bin") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let col = Collection::load(&path)
                    .with_context(|| format!("failed to load collection from {}", path.display()))?;
                let mut cs = CollectionState::new(col);
                cs.rebuild(&gpu);
                registry_map.insert(name, cs);
            }
        }
        println!("beam: loaded {} collection(s) from {}", registry_map.len(), dir.display());
    }

    let registry = Arc::new(RwLock::new(registry_map));
    let auth = Arc::new(auth_config.verifier());

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("failed to bind {addr}: {e}"))?;
    let bound = listener.local_addr()?;
    println!("beam serving on http://{bound}");

    let shutdown = service_http::wait_shutdown_signal();
    serve_on(listener, router_with_state(registry, gpu, data_dir, auth), shutdown).await;
    Ok(())
}
// </HANDWRITE>
