// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Wire types for the public HTTP API.
//!
//! These structs serialize to and from the JSON shapes documented in
//! `projects/lumen/README.md`. They power the live router and the
//! OpenAPI schema served at `GET /openapi.json` — so they are the
//! single source of truth consumers integrate against.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

// ---------------------------------------------------------------------------
// Schema (DDL)
// ---------------------------------------------------------------------------

/// `PUT /collections/{id}` body.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct CreateCollectionRequest {
    pub fields: BTreeMap<String, FieldSpec>,
}

/// `PUT /collections/{id}` response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct CreateCollectionResponse {
    pub collection_id: String,
    pub version: u32,
    pub fields_count: u32,
}

/// One field declaration inside a collection schema.
///
/// The `dim` / `metric` / `backend` / `quantize` fields are only
/// meaningful when `field_type == FieldType::Vector`; they are
/// rejected by schema validation on any other field type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct FieldSpec {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Analyzer for `text`. Ignored on other types. Defaults to
    /// `whitespace_lower` when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analyzer: Option<Analyzer>,
    /// Convenience flag: `{type: "keyword", multi: true}` is sugar for
    /// `{type: "set"}`. Normalized at schema-validation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi: Option<bool>,
    /// Vector dimensionality. Required when `type == "vector"`,
    /// rejected otherwise. Immutable for the field's lifetime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dim: Option<u32>,
    /// Vector distance metric. Required when `type == "vector"`,
    /// rejected otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<VectorMetric>,
    /// Vector index backend. Defaults to `hnsw-cpu` when omitted on a
    /// vector field, rejected on any other field type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<VectorBackend>,
    /// Optional quantization scheme — only meaningful on vector
    /// fields. `sq` enables transparent scalar quantization (f32→u8);
    /// `pq` is reserved for a future product-quantization landing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantize: Option<VectorQuantize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum FieldType {
    Text,
    Keyword,
    Number,
    Set,
    Vector,
    /// 64-bit perceptual/structural hash (pHash / dHash / SimHash / b-bit
    /// MinHash). The caller computes the hash; lumen indexes the bits and
    /// answers Hamming-distance queries. Wire value is a hex string.
    Hash,
}

/// Distance metric for `FieldType::Vector`. Wire form is snake_case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum VectorMetric {
    Cosine,
    Dot,
    L2,
}

/// Index backend for `FieldType::Vector`. Wire forms are
/// `hnsw-cpu` / `flat-cpu` (kebab-case).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum VectorBackend {
    /// Approximate HNSW graph (CPU). Sub-linear, recall < 1.
    HnswCpu,
    /// Exact CPU brute-force: no index/build, parallel + vectorized full scan.
    /// 100% recall; for moderate N it beats both an approximate index's build
    /// cost and a single-threaded exact scan, and is the default CPU choice when
    /// exactness matters.
    FlatCpu,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
impl Default for VectorBackend {
    fn default() -> Self {
        Self::HnswCpu
    }
}

/// Quantization scheme for `FieldType::Vector`.
///
/// `sq` enables transparent scalar quantization (f32 stored as u8).
/// `pq` is reserved for a future product-quantization landing and
/// is not yet implemented — declaring it will be rejected at schema
/// time until the backing codec ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum VectorQuantize {
    Sq,
    Pq,
}

/// Resolved vector field configuration. Built from a `FieldSpec`
/// once schema validation has confirmed all required slots are present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct VectorSpec {
    pub dim: u32,
    pub metric: VectorMetric,
    pub backend: VectorBackend,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantize: Option<VectorQuantize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum Analyzer {
    WhitespaceLower,
    Jieba,
    Ngram,
}

// ---------------------------------------------------------------------------
// Index (write)
// ---------------------------------------------------------------------------

/// `POST /collections/{id}/index` body.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct IndexRequest {
    pub items: Vec<IndexItem>,
    /// Optional idempotency key. Repeated requests within 5 min are
    /// silently deduplicated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct IndexItem {
    pub external_id: String,
    pub field: String,
    pub value: FieldValue,
    /// Optional external version for last-write-wins. When set, lumen keeps the
    /// highest version per `(external_id, field)` and drops strictly-older
    /// writes (cf. Elasticsearch `version_type=external`), so out-of-order
    /// delivery cannot clobber a newer value. When absent, the write applies in
    /// arrival order.
    /// @spec projects/lumen/tech-design/logic/external-version-lww-optional-version-on-indexitem-drop-stale-pe.md
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
}

/// Polymorphic field value. Validated against the declared `FieldType`
/// at index time; mismatches return 422.
///
/// On the wire, `Vector` is a plain JSON `[f32]` array — `serde(untagged)`
/// resolves the variant by JSON shape (string vs number vs list-of-string
/// vs list-of-number).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum FieldValue {
    String(String),
    Number(f64),
    Vector(Vec<f32>),
    StringList(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct IndexResponse {
    pub indexed: u32,
    pub bytes_written: BTreeMap<String, u64>,
    pub shard_lag_ms: u64,
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

/// `POST /collections/{id}/search` body.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct SearchRequest {
    pub query: QueryNode,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Sort results by one or more fields instead of by relevance score.
    /// When absent, results are ranked by score (BM25 / constant) then
    /// external_id. Number and keyword fields are sortable (up to 2 keys);
    /// single number-field sorts use the keyset planner, keyword and composite
    /// sorts use the materialized fallback. Rows missing a sort-key value
    /// follow the per-key `missing` mode: `exclude` (the default) drops them
    /// from the page and from `total`; `first`/`last` keep them — placed
    /// before/after all present values and counted in `total`, like SQL
    /// `NULLS FIRST`/`NULLS LAST`. A `sort`
    /// cannot be combined with an offset cursor — that returns 400; page a
    /// sorted result with the keyset cursor returned in the response, or
    /// over-fetch and slice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<Vec<SortSpec>>,
    /// Whether to compute the exact total match count. Defaults to `true`
    /// (back-compat). When `false`, the planner may early-terminate and
    /// `total` becomes a lower bound (≥ the returned page size).
    #[serde(default = "default_track_total")]
    pub track_total: bool,
    /// Collapse (field-collapse / group-by) on a keyword field: return ONE hit
    /// per distinct value of this field, scored by the MAX member score, ranked
    /// over groups. `hit.external_id` becomes the collapse value, `total` the
    /// distinct-group count. Used for nested `group` search: filter the child
    /// collection, collapse by `parent_row_id` → distinct matching parents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collapse: Option<String>,
}

fn default_limit() -> u32 {
    20
}

fn default_track_total() -> bool {
    true
}

/// One sort key. `order` defaults to ascending.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct SortSpec {
    pub field: String,
    #[serde(default)]
    pub order: SortOrder,
    /// How to treat rows that have no value for this sort key. Default
    /// `exclude` keeps today's behavior (such rows are dropped from results and
    /// from `total`). `first`/`last` keep them, placed before/after the rows
    /// that do have a value, and count them in `total`.
    /// @spec projects/lumen/tech-design/logic/sort-missing-value-handling-opt-in-missing-first-last-exclude-an.md
    #[serde(default)]
    pub missing: SortMissing,
}

/// Placement of rows missing a value for a sort key (SQL NULLS FIRST/LAST).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/logic/sort-missing-value-handling-opt-in-missing-first-last-exclude-an.md
pub enum SortMissing {
    /// Drop rows that lack a value for this key (default; today's behavior).
    #[default]
    Exclude,
    /// Place rows lacking a value before all rows that have one.
    First,
    /// Place rows lacking a value after all rows that have one.
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

/// A search query node. Externally tagged on the wire:
///
/// ```json
/// { "match": { "field": "bio", "text": "engineer" } }
/// { "term":  { "field": "tags", "value": "rust" } }
/// { "knn":   { "field": "embedding", "vector": [0.1, ...], "k": 10 } }
/// { "and":   [ {...}, {...} ] }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum QueryNode {
    Match(MatchQuery),
    Term(TermQuery),
    Terms(TermsQuery),
    /// Filter to a set of external_ids (#182). Wire: `{"ids": {"values":[...]}}`.
    Ids(IdsQuery),
    Range(RangeQuery),
    Knn(KnnQuery),
    And(Vec<QueryNode>),
    Or(Vec<QueryNode>),
    Not(Box<QueryNode>),
    /// Nested as a first-class clause: a parent matches if its child collection
    /// has ≥1 doc (linked by `field` = parent_row_id) matching the sub-`query`.
    /// Evaluates to the set of parent docids → composes under and/or/not like
    /// any other clause. Drives data-table `group` search inside arbitrary
    /// boolean trees. Wire: `{"has_child": {"collection","field","query":{...}}}`.
    #[serde(rename = "has_child")]
    HasChild(HasChildQuery),
    /// Perceptual/structural near-duplicate search over a `hash` field: every
    /// doc whose 64-bit hash is within `max_distance` Hamming bits of `hash`,
    /// scored by similarity (closer = higher). Wire:
    /// `{"hamming": {"field","hash":"<hex>","max_distance":N}}`.
    #[serde(rename = "hamming")]
    Hamming(HammingQuery),
    /// Reciprocal Rank Fusion: run each sub-query, rank its hits by score
    /// descending, and fuse into one ranking *by rank* — `score(d) = Σ_i 1/(k +
    /// rank_i(d))` over the sub-queries that returned `d` (rank is 1-based).
    /// Because it fuses ranks, BM25 and cosine score scales need no
    /// normalisation. This is hybrid lexical+semantic retrieval. Put any filter
    /// **inside each leg** (e.g. `{"and":[{"knn":…},{"term":…}]}`) so the kNN leg
    /// stays filter-correct. Wire: `{"rrf": {"queries":[{…},{…}], "k":60}}`.
    #[serde(rename = "rrf")]
    Rrf(RrfQuery),
    /// Non-null / "has a value" predicate over any indexed field: the set of docs
    /// that have ≥1 value in `field`. Composes under and/or/not like any filter.
    /// Drives data-table "is not empty" filters (the inverse is `{"not":{"exists":…}}`).
    /// Wire: `{"exists": {"field": "email"}}`.
    #[serde(rename = "exists")]
    Exists(ExistsQuery),
    /// Docs whose `field` value is SHARED — the value occurs in ≥ `min_group_size`
    /// docs (default 2). The boolean-composable form of `/duplicates`: drops into
    /// and/or/not (e.g. "duplicated email AND city=Taipei"). keyword/number/set
    /// only. Wire: `{"duplicated": {"field":"email","min_group_size":2}}`.
    #[serde(rename = "duplicated")]
    Duplicated(DuplicatedQuery),
}

/// `exists` predicate (see [`QueryNode::Exists`]).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct ExistsQuery {
    pub field: String,
}

/// `duplicated` predicate (see [`QueryNode::Duplicated`]).
/// Reuses `default_min_group_size` (defined with `DuplicatesRequest`).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct DuplicatedQuery {
    pub field: String,
    /// Minimum group size to count as duplicated (default 2).
    #[serde(default = "default_min_group_size")]
    pub min_group_size: u32,
}

/// Reciprocal Rank Fusion query (see [`QueryNode::Rrf`]).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct RrfQuery {
    /// Sub-queries whose rankings are fused (≥1; typically a `knn` + a `match`).
    pub queries: Vec<QueryNode>,
    /// RRF rank constant — larger flattens the weighting. Default 60.
    #[serde(default = "default_rrf_k")]
    pub k: u32,
}

fn default_rrf_k() -> u32 {
    60
}

/// Hamming near-duplicate query over a `hash` field (see [`QueryNode::Hamming`]).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct HammingQuery {
    pub field: String,
    /// The query hash as a 64-bit hex string (optionally `0x`-prefixed).
    pub hash: String,
    /// Inclusive maximum Hamming distance (0..=64) for a match.
    pub max_distance: u32,
}

/// `has_child` sub-query (see [`QueryNode::HasChild`]).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct HasChildQuery {
    /// The child collection to evaluate `query` against.
    pub collection: String,
    /// The child's keyword field holding the parent's external_id.
    pub field: String,
    pub query: Box<QueryNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct MatchQuery {
    pub field: String,
    pub text: String,
    #[serde(default = "default_match_op")]
    pub op: MatchOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub enum MatchOp {
    And,
    Or,
}

fn default_match_op() -> MatchOp {
    MatchOp::And
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct TermQuery {
    pub field: String,
    pub value: FieldValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct TermsQuery {
    pub field: String,
    pub values: Vec<FieldValue>,
}

/// `ids` query node (#182): filter to a set of external_ids. Each id is resolved
/// through the collection interner to a docid (unknown ids are skipped). It is
/// constant-scored and composes under and/or/not like term/terms. Removes the
/// need to index a redundant row-id keyword field for `row_id_in`.
/// @spec projects/lumen/tech-design/logic/native-ids-query-node-filter-by-external-id-set.md
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IdsQuery {
    pub values: Vec<String>,
}

/// kNN vector search node. Returns the `k` external_ids closest to
/// `vector` under the field's declared metric. Scores are the negated
/// distance — higher = better, consistent with BM25 / term scoring.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct KnnQuery {
    pub field: String,
    pub vector: Vec<f32>,
    pub k: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct RangeQuery {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gt: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gte: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lt: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lte: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct SearchHit {
    pub external_id: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    pub took_ms: u64,
    /// Server-side engine time in microseconds — the same measurement as
    /// `took_ms` at sub-millisecond resolution, so callers can see how fast the
    /// engine answered when `took_ms` rounds to 0.
    #[serde(default)]
    pub took_us: u64,
}

// ---------------------------------------------------------------------------
// Duplicates
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct DuplicatesRequest {
    pub field: String,
    #[serde(default = "default_min_group_size")]
    pub min_group_size: u32,
    #[serde(default = "default_dup_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_min_group_size() -> u32 {
    2
}
fn default_dup_limit() -> u32 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct DuplicateGroup {
    pub value: serde_json::Value,
    pub external_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct DuplicatesResponse {
    pub groups: Vec<DuplicateGroup>,
    pub truncated: bool,
    pub took_ms: u64,
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

/// Engine-level metadata about one collection.
///
/// Everything here describes **the index**, not the caller's data —
/// lumen is a search specialist, not an analytics engine. For data
/// aggregations (group-by / histogram / percentile / pipeline), pair
/// lumen with an OLAP store (ClickHouse / Druid / BigQuery / DuckDB)
/// and dual-write.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct StatsResponse {
    /// Distinct `external_id` count in this collection.
    pub documents_indexed: u64,
    /// Per-field engine metadata.
    pub fields: BTreeMap<String, FieldStats>,
    /// Aggregate storage footprint.
    pub storage: StorageStats,
    /// Read-cache health (`moka` byte-weighted LRU on the LSM path).
    pub cache: CacheStats,
    /// Most recent successful write into this collection, RFC 3339
    /// (UTC). `None` when no write has landed since the last reboot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_indexed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct FieldStats {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Distinct terms / values / elements / vectors (depending on type).
    pub unique_terms: u64,
    /// Bytes the engine attributes to this field's indexes.
    pub bytes: u64,
    /// Mean tokens per document, only populated on `text` fields.
    /// Exposes the BM25 length-normalization denominator so callers
    /// can reason about scoring stability without dumping internals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_doc_len: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct StorageStats {
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct CacheStats {
    /// Hit ratio on the posting-list cache. `1.0` when no cache layer
    /// is attached (in-memory engine has no need for one).
    pub posting_hit_ratio: f32,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
pub struct ApiError {
    pub error: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Normalization
// ---------------------------------------------------------------------------

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-types-rs.md#source
impl FieldSpec {
    /// Normalize sugar: `{type: "keyword", multi: true}` → `{type: "set"}`.
    /// Sets a default analyzer on `text` if absent. Fills in a default
    /// `hnsw-cpu` backend on a vector field when none is declared.
    pub fn normalize(mut self) -> Self {
        if matches!(self.field_type, FieldType::Keyword) && self.multi == Some(true) {
            self.field_type = FieldType::Set;
            self.multi = None;
        }
        if matches!(self.field_type, FieldType::Text) && self.analyzer.is_none() {
            self.analyzer = Some(Analyzer::WhitespaceLower);
        }
        if matches!(self.field_type, FieldType::Vector) && self.backend.is_none() {
            self.backend = Some(VectorBackend::default());
        }
        self
    }

    /// Resolve the vector-specific sub-shape if the field is a vector
    /// field. Returns `None` for non-vector fields. Returns an error
    /// when the field is declared as a vector but is missing required
    /// `dim` / `metric` slots.
    pub fn vector_spec(&self) -> anyhow::Result<Option<VectorSpec>> {
        if !matches!(self.field_type, FieldType::Vector) {
            return Ok(None);
        }
        let dim = self
            .dim
            .ok_or_else(|| anyhow::anyhow!("vector field is missing `dim`"))?;
        let metric = self
            .metric
            .ok_or_else(|| anyhow::anyhow!("vector field is missing `metric`"))?;
        let backend = self.backend.unwrap_or_default();
        if dim == 0 {
            anyhow::bail!("vector field `dim` must be > 0");
        }
        if matches!(self.quantize, Some(VectorQuantize::Pq)) {
            anyhow::bail!("product quantization (`pq`) is not yet implemented");
        }
        Ok(Some(VectorSpec {
            dim,
            metric,
            backend,
            quantize: self.quantize,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fs(t: FieldType) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    #[test]
    fn field_spec_keyword_multi_becomes_set() {
        let spec = FieldSpec {
            multi: Some(true),
            ..fs(FieldType::Keyword)
        }
        .normalize();
        assert_eq!(spec.field_type, FieldType::Set);
        assert!(spec.multi.is_none());
    }

    #[test]
    fn field_spec_text_gets_default_analyzer() {
        let spec = fs(FieldType::Text).normalize();
        assert_eq!(spec.analyzer, Some(Analyzer::WhitespaceLower));
    }

    #[test]
    fn field_spec_vector_defaults_backend_to_hnsw_cpu() {
        let spec = FieldSpec {
            dim: Some(128),
            metric: Some(VectorMetric::Cosine),
            ..fs(FieldType::Vector)
        }
        .normalize();
        assert_eq!(spec.backend, Some(VectorBackend::HnswCpu));
    }

    #[test]
    fn field_spec_vector_spec_wire_round_trip() {
        let j =
            r#"{"type":"vector","dim":768,"metric":"cosine","backend":"hnsw-cpu","quantize":"sq"}"#;
        let s: FieldSpec = serde_json::from_str(j).unwrap();
        assert!(matches!(s.field_type, FieldType::Vector));
        let vs = s.vector_spec().unwrap().unwrap();
        assert_eq!(vs.dim, 768);
        assert!(matches!(vs.metric, VectorMetric::Cosine));
        assert!(matches!(vs.backend, VectorBackend::HnswCpu));
        assert!(matches!(vs.quantize, Some(VectorQuantize::Sq)));
    }

    #[test]
    fn field_spec_vector_missing_dim_rejected() {
        let bad = FieldSpec {
            metric: Some(VectorMetric::L2),
            ..fs(FieldType::Vector)
        };
        assert!(bad.vector_spec().is_err());
    }

    #[test]
    fn vector_value_round_trip_as_float_array() {
        let v: FieldValue = serde_json::from_str("[0.1, 0.2, 0.3]").unwrap();
        match v {
            FieldValue::Vector(xs) => {
                assert_eq!(xs.len(), 3);
                assert!((xs[0] - 0.1).abs() < 1e-6);
            }
            _ => panic!("expected Vector, got {v:?}"),
        }
    }

    #[test]
    fn query_node_knn_round_trip() {
        let j = r#"{"knn":{"field":"e","vector":[0.1,0.2,0.3],"k":5}}"#;
        let q: QueryNode = serde_json::from_str(j).unwrap();
        match q {
            QueryNode::Knn(k) => {
                assert_eq!(k.field, "e");
                assert_eq!(k.k, 5);
                assert_eq!(k.vector.len(), 3);
            }
            _ => panic!("expected Knn, got {q:?}"),
        }
    }

    #[test]
    fn query_node_match_serializes_externally_tagged() {
        let q = QueryNode::Match(MatchQuery {
            field: "bio".into(),
            text: "rust".into(),
            op: MatchOp::And,
        });
        let j = serde_json::to_string(&q).unwrap();
        assert!(j.contains("\"match\""), "got: {j}");
        assert!(j.contains("\"field\":\"bio\""));
    }

    #[test]
    fn query_node_and_round_trip() {
        let j = r#"{"and":[{"term":{"field":"tags","value":"rust"}},{"range":{"field":"age","gte":25,"lt":40}}]}"#;
        let q: QueryNode = serde_json::from_str(j).unwrap();
        match q {
            QueryNode::And(ref children) => assert_eq!(children.len(), 2),
            _ => panic!("expected And, got {q:?}"),
        }
    }

    #[test]
    fn field_value_polymorphic() {
        let s: FieldValue = serde_json::from_str(r#""hello""#).unwrap();
        assert!(matches!(s, FieldValue::String(_)));
        let n: FieldValue = serde_json::from_str("42").unwrap();
        assert!(matches!(n, FieldValue::Number(_)));
        let l: FieldValue = serde_json::from_str(r#"["a","b"]"#).unwrap();
        assert!(matches!(l, FieldValue::StringList(_)));
    }
}
// CODEGEN-END
