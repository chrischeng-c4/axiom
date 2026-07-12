// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Shard routing.
//!
//! The stable routing contract is virtual-bucket based:
//! `bucket = hash(collection_id, routing_key || external_id) % N`, then a
//! versioned bucket-to-physical-shard map decides ownership. That keeps
//! `shardCount` from becoming a permanent `hash % shardCount` data contract,
//! which is required for operator-managed shard splits.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::time::Instant;

use std::sync::Arc;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures::future::try_join_all;
use rayon::prelude::*;

use crate::api::{SearchBackend, WriteBackend};
use crate::coordinator::WriteCoordinator;
use crate::log_entry::RaftLogEntry;
use crate::storage::{ApplyOutcome, DropOutcome, Engine};
use crate::types::{
    CreateCollectionRequest, CreateCollectionResponse, IndexRequest, IndexResponse, ReplaceDocItem,
    ReplaceDocResult, ReplaceDocsRequest, ReplaceDocsResponse, SearchHit, SearchRequest,
    SearchResponse, SortOrder,
};

pub const DEFAULT_VIRTUAL_BUCKET_COUNT: u32 = 4096;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub fn shard_index(collection_id: &str, shard_count: u32) -> u32 {
    let map = VirtualBucketShardMap::balanced(0, DEFAULT_VIRTUAL_BUCKET_COUNT, shard_count)
        .expect("shard_count must be > 0");
    map.route_key(collection_id, collection_id).shard
}

/// Row/document routing for a sharded local serving node. Collection-level
/// routing now uses the same virtual-bucket map as cluster routing, with
/// `external_id` as the default routing key. That lets one large collection
/// spread across shards while each document remains owned by exactly one shard.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub fn document_shard_index(collection_id: &str, external_id: &str, shard_count: usize) -> usize {
    let physical = u32::try_from(shard_count).expect("shard_count must fit in u32");
    let map = VirtualBucketShardMap::balanced(0, DEFAULT_VIRTUAL_BUCKET_COUNT, physical)
        .expect("shard_count must be > 0");
    map.route_document(collection_id, None, external_id).shard as usize
}

fn route_hash(collection_id: &str, key: &str) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(collection_id.as_bytes());
    hasher.update(&[0]);
    hasher.update(key.as_bytes());
    hasher.finalize()
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub struct ShardRoute {
    pub map_version: u64,
    pub virtual_bucket_count: u32,
    pub physical_shard_count: u32,
    pub bucket: u32,
    pub shard: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub enum SearchShardTarget {
    All,
    One(ShardRoute),
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub struct VirtualBucketShardMap {
    version: u64,
    assignments: Arc<Vec<u32>>,
    physical_shard_count: u32,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
impl VirtualBucketShardMap {
    pub fn balanced(
        version: u64,
        virtual_bucket_count: u32,
        physical_shard_count: u32,
    ) -> Result<Self> {
        if virtual_bucket_count == 0 {
            bail!("virtual_bucket_count must be > 0");
        }
        if physical_shard_count == 0 {
            bail!("physical_shard_count must be > 0");
        }
        let assignments = (0..virtual_bucket_count)
            .map(|bucket| bucket % physical_shard_count)
            .collect();
        Self::new(version, assignments, physical_shard_count)
    }

    pub fn new(version: u64, assignments: Vec<u32>, physical_shard_count: u32) -> Result<Self> {
        if assignments.is_empty() {
            bail!("shard map must contain at least one virtual bucket");
        }
        if physical_shard_count == 0 {
            bail!("physical_shard_count must be > 0");
        }
        for &shard in &assignments {
            if shard >= physical_shard_count {
                bail!(
                    "bucket assignment points to shard {shard}, but physical_shard_count is {physical_shard_count}"
                );
            }
        }
        Ok(Self {
            version,
            assignments: Arc::new(assignments),
            physical_shard_count,
        })
    }

    pub fn single() -> Self {
        Self::new(0, vec![0], 1).expect("single shard map is valid")
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn virtual_bucket_count(&self) -> u32 {
        self.assignments.len() as u32
    }

    pub fn physical_shard_count(&self) -> u32 {
        self.physical_shard_count
    }

    pub fn assignment_for_bucket(&self, bucket: u32) -> Option<u32> {
        self.assignments.get(bucket as usize).copied()
    }

    pub fn route_document(
        &self,
        collection_id: &str,
        routing_key: Option<&str>,
        external_id: &str,
    ) -> ShardRoute {
        self.route_key(collection_id, routing_key.unwrap_or(external_id))
    }

    pub fn route_key(&self, collection_id: &str, routing_key: &str) -> ShardRoute {
        let bucket = route_hash(collection_id, routing_key) % self.virtual_bucket_count();
        let shard = self.assignments[bucket as usize];
        ShardRoute {
            map_version: self.version,
            virtual_bucket_count: self.virtual_bucket_count(),
            physical_shard_count: self.physical_shard_count,
            bucket,
            shard,
        }
    }

    pub fn search_target(
        &self,
        collection_id: &str,
        routing_key: Option<&str>,
    ) -> SearchShardTarget {
        match routing_key {
            Some(key) => SearchShardTarget::One(self.route_key(collection_id, key)),
            None => SearchShardTarget::All,
        }
    }

    /// Target map for growing this map by exactly one physical shard, moving
    /// the minimum number of virtual buckets. From each existing shard, the
    /// lowest-numbered `buckets_on_that_shard / new_physical_shard_count`
    /// buckets move to the new shard (appended at index
    /// `physical_shard_count`); no bucket ever moves directly between two
    /// existing shards. That keeps a single split a bounded, per-source-shard
    /// migration (one batch stream per old shard into the new shard) rather
    /// than a full cluster-wide rebalance.
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
    pub fn split_one_shard(&self, new_version: u64) -> Result<Self> {
        let new_physical_shard_count = self
            .physical_shard_count
            .checked_add(1)
            .context("physical_shard_count overflow computing shard split")?;
        let new_shard = self.physical_shard_count;

        let mut buckets_by_shard: Vec<Vec<u32>> =
            vec![Vec::new(); self.physical_shard_count as usize];
        for (bucket, &shard) in self.assignments.iter().enumerate() {
            buckets_by_shard[shard as usize].push(bucket as u32);
        }

        let mut assignments = (*self.assignments).clone();
        for buckets in &buckets_by_shard {
            let move_count = buckets.len() / new_physical_shard_count as usize;
            for &bucket in buckets.iter().take(move_count) {
                assignments[bucket as usize] = new_shard;
            }
        }

        Self::new(new_version, assignments, new_physical_shard_count)
    }
}

/// DNS for a given shard's stable client entry (any replica will do —
/// the server forwards writes internally).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub fn shard_host(prefix: &str, shard: u32, headless_service: &str) -> String {
    format!("{prefix}-{shard}.{headless_service}")
}

#[derive(Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub struct EngineShardSearch {
    shards: Arc<Vec<Arc<Engine>>>,
    shard_map: VirtualBucketShardMap,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
impl EngineShardSearch {
    pub fn new(shards: Vec<Arc<Engine>>) -> Self {
        let shard_count = u32::try_from(shards.len()).expect("shard count must fit in u32");
        let shard_map =
            VirtualBucketShardMap::balanced(0, DEFAULT_VIRTUAL_BUCKET_COUNT, shard_count.max(1))
                .expect("balanced shard map");
        Self::new_with_shard_map(shards, shard_map)
    }

    pub fn new_with_shard_map(shards: Vec<Arc<Engine>>, shard_map: VirtualBucketShardMap) -> Self {
        Self {
            shards: Arc::new(shards),
            shard_map,
        }
    }

    pub fn len(&self) -> usize {
        self.shards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.shards.is_empty()
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
impl SearchBackend for EngineShardSearch {
    fn search(&self, collection_id: &str, req: SearchRequest) -> Result<SearchResponse> {
        let selected_shards: Vec<Arc<Engine>> = match self
            .shard_map
            .search_target(collection_id, req.routing_key.as_deref())
        {
            SearchShardTarget::All => self.shards.iter().cloned().collect(),
            SearchShardTarget::One(route) => {
                let Some(engine) = self.shards.get(route.shard as usize) else {
                    bail!("shard map routed to missing shard {}", route.shard);
                };
                vec![engine.clone()]
            }
        };
        search_shards_parallel(
            collection_id,
            req,
            selected_shards.as_slice(),
            |engine, collection_id, req| Ok(engine.search(collection_id, req)?),
            |hit, field| {
                self.shards.iter().find_map(|engine| {
                    engine
                        .number_value_for_external_id(collection_id, &hit.external_id, field)
                        .ok()
                        .flatten()
                })
            },
        )
    }
}

#[derive(Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub struct EngineShardWrite {
    writers: Arc<Vec<Arc<WriteCoordinator>>>,
    shard_map: VirtualBucketShardMap,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
impl EngineShardWrite {
    pub fn new(writers: Vec<Arc<WriteCoordinator>>) -> Self {
        let shard_count = u32::try_from(writers.len()).expect("shard count must fit in u32");
        let shard_map =
            VirtualBucketShardMap::balanced(0, DEFAULT_VIRTUAL_BUCKET_COUNT, shard_count.max(1))
                .expect("balanced shard map");
        Self::new_with_shard_map(writers, shard_map)
    }

    pub fn new_with_shard_map(
        writers: Vec<Arc<WriteCoordinator>>,
        shard_map: VirtualBucketShardMap,
    ) -> Self {
        Self {
            writers: Arc::new(writers),
            shard_map,
        }
    }

    pub fn len(&self) -> usize {
        self.writers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.writers.is_empty()
    }

    fn require_shards(&self) -> Result<()> {
        if self.writers.is_empty() {
            bail!("sharded write backend has no shards");
        }
        Ok(())
    }
}

#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
impl WriteBackend for EngineShardWrite {
    async fn create_collection(
        &self,
        collection_id: String,
        req: CreateCollectionRequest,
    ) -> Result<CreateCollectionResponse> {
        self.require_shards()?;
        let outcomes = try_join_all(self.writers.iter().map(|writer| {
            let writer = writer.clone();
            let collection_id = collection_id.clone();
            let req = req.clone();
            async move {
                writer
                    .submit(RaftLogEntry::CreateCollection { collection_id, req })
                    .await
            }
        }))
        .await?;

        let mut first: Option<CreateCollectionResponse> = None;
        for outcome in outcomes {
            match outcome {
                ApplyOutcome::Created(resp) => {
                    if let Some(existing) = &first {
                        if existing.version != resp.version
                            || existing.fields_count != resp.fields_count
                        {
                            bail!("shard collection-create responses diverged");
                        }
                    } else {
                        first = Some(resp);
                    }
                }
                other => bail!("unexpected apply outcome: {other:?}"),
            }
        }
        first.ok_or_else(|| anyhow::anyhow!("sharded create produced no responses"))
    }

    async fn drop_collection(&self, collection_id: String, force: bool) -> Result<DropOutcome> {
        self.require_shards()?;
        let outcomes = try_join_all(self.writers.iter().map(|writer| {
            let writer = writer.clone();
            let collection_id = collection_id.clone();
            async move {
                writer
                    .submit(RaftLogEntry::DropCollection {
                        collection_id,
                        force,
                    })
                    .await
            }
        }))
        .await?;

        let mut merged = DropOutcome::NotFound;
        for outcome in outcomes {
            let ApplyOutcome::Dropped(outcome) = outcome else {
                bail!("unexpected apply outcome: {outcome:?}");
            };
            merged = match (merged, outcome) {
                (DropOutcome::Physical, _) | (_, DropOutcome::Physical) => DropOutcome::Physical,
                (DropOutcome::Marked, _) | (_, DropOutcome::Marked) => DropOutcome::Marked,
                (DropOutcome::AlreadyMarked, _) | (_, DropOutcome::AlreadyMarked) => {
                    DropOutcome::AlreadyMarked
                }
                (DropOutcome::NotFound, DropOutcome::NotFound) => DropOutcome::NotFound,
            };
        }
        Ok(merged)
    }

    async fn index(&self, collection_id: String, req: IndexRequest) -> Result<IndexResponse> {
        self.require_shards()?;
        let mut shard_reqs: Vec<IndexRequest> = (0..self.writers.len())
            .map(|_| IndexRequest {
                items: Vec::new(),
                request_id: req.request_id.clone(),
            })
            .collect();

        for item in req.items {
            let shard = self
                .shard_map
                .route_document(&collection_id, None, &item.external_id)
                .shard as usize;
            shard_reqs[shard].items.push(item);
        }

        let has_items = shard_reqs.iter().any(|req| !req.items.is_empty());
        let mut futures = Vec::new();
        for (shard, req) in shard_reqs.into_iter().enumerate() {
            if has_items && req.items.is_empty() {
                continue;
            }
            let writer = self.writers[shard].clone();
            let collection_id = collection_id.clone();
            futures.push(async move {
                writer
                    .submit(RaftLogEntry::Index { collection_id, req })
                    .await
            });
            if !has_items {
                break;
            }
        }

        let outcomes = try_join_all(futures).await?;
        let mut indexed = 0u32;
        let mut bytes_written = BTreeMap::new();
        let mut shard_lag_ms = 0u64;
        for outcome in outcomes {
            let ApplyOutcome::Indexed(resp) = outcome else {
                bail!("unexpected apply outcome: {outcome:?}");
            };
            indexed = indexed.saturating_add(resp.indexed);
            shard_lag_ms = shard_lag_ms.max(resp.shard_lag_ms);
            for (field, bytes) in resp.bytes_written {
                *bytes_written.entry(field).or_insert(0) += bytes;
            }
        }
        Ok(IndexResponse {
            indexed,
            bytes_written,
            shard_lag_ms,
        })
    }

    async fn replace_docs(
        &self,
        collection_id: String,
        req: ReplaceDocsRequest,
    ) -> Result<ReplaceDocsResponse> {
        self.require_shards()?;
        let total = req.docs.len();
        let mut shard_reqs: Vec<Vec<ReplaceDocItem>> =
            (0..self.writers.len()).map(|_| Vec::new()).collect();
        let mut shard_positions: Vec<Vec<usize>> =
            (0..self.writers.len()).map(|_| Vec::new()).collect();

        for (idx, item) in req.docs.into_iter().enumerate() {
            let shard = self
                .shard_map
                .route_document(&collection_id, None, &item.external_id)
                .shard as usize;
            shard_positions[shard].push(idx);
            shard_reqs[shard].push(item);
        }

        let mut futures = Vec::new();
        let mut shard_order = Vec::new();
        for (shard, docs) in shard_reqs.into_iter().enumerate() {
            if docs.is_empty() {
                continue;
            }
            let writer = self.writers[shard].clone();
            let collection_id = collection_id.clone();
            shard_order.push(shard);
            futures.push(async move {
                writer
                    .submit(RaftLogEntry::ReplaceDocs {
                        collection_id,
                        req: ReplaceDocsRequest { docs },
                    })
                    .await
            });
        }

        let outcomes = try_join_all(futures).await?;
        // Reassemble in the caller's original `req.docs` order — shard
        // fan-out only preserves order *within* a shard, so each result
        // is placed back at its origin index rather than appended in
        // shard-completion order.
        let mut results: Vec<Option<ReplaceDocResult>> = (0..total).map(|_| None).collect();
        for (shard, outcome) in shard_order.into_iter().zip(outcomes) {
            let ApplyOutcome::Replaced(resp) = outcome else {
                bail!("unexpected apply outcome: {outcome:?}");
            };
            for (pos, result) in shard_positions[shard].iter().zip(resp.results) {
                results[*pos] = Some(result);
            }
        }
        let results: Vec<ReplaceDocResult> = results
            .into_iter()
            .map(|r| r.expect("every original index assigned exactly one shard result"))
            .collect();
        Ok(ReplaceDocsResponse { results })
    }

    async fn delete(
        &self,
        collection_id: String,
        external_id: String,
        field: Option<String>,
    ) -> Result<()> {
        self.require_shards()?;
        let shard = self
            .shard_map
            .route_document(&collection_id, None, &external_id)
            .shard as usize;
        match self.writers[shard]
            .submit(RaftLogEntry::Delete {
                collection_id,
                external_id,
                field,
            })
            .await?
        {
            ApplyOutcome::Deleted => Ok(()),
            other => bail!("unexpected apply outcome: {other:?}"),
        }
    }

    async fn drop_field(&self, collection_id: String, field_name: String) -> Result<u32> {
        self.require_shards()?;
        let outcomes = try_join_all(self.writers.iter().map(|writer| {
            let writer = writer.clone();
            let collection_id = collection_id.clone();
            let field_name = field_name.clone();
            async move {
                writer
                    .submit(RaftLogEntry::DropField {
                        collection_id,
                        field_name,
                    })
                    .await
            }
        }))
        .await?;

        let mut version = None;
        for outcome in outcomes {
            let ApplyOutcome::FieldChanged(v) = outcome else {
                bail!("unexpected apply outcome: {outcome:?}");
            };
            if let Some(existing) = version {
                if existing != v {
                    bail!("shard drop-field versions diverged");
                }
            } else {
                version = Some(v);
            }
        }
        version.ok_or_else(|| anyhow::anyhow!("sharded drop-field produced no responses"))
    }
}

/// Query sealed/local shards in parallel and merge the top page into the same
/// response shape as a single-engine search.
///
/// `sort_value` resolves numeric sort keys for returned hits. It exists because
/// [`SearchHit`] intentionally carries only `(external_id, score)` today; a
/// production sharded router can resolve values from shard-local metadata, while
/// the scale bench derives deterministic corpus values without widening the
/// public response type.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub fn search_shards_parallel<S, F, K>(
    collection_id: &str,
    req: SearchRequest,
    shards: &[S],
    search: F,
    sort_value: K,
) -> Result<SearchResponse>
where
    S: Sync,
    F: Fn(&S, &str, SearchRequest) -> Result<SearchResponse> + Sync,
    K: Fn(&SearchHit, &str) -> Option<f64> + Sync,
{
    let start = Instant::now();
    let offset = req.cursor.as_deref().and_then(parse_cursor).unwrap_or(0) as usize;
    let limit = req.limit as usize;
    let mut shard_req = req.clone();
    shard_req.cursor = None;
    shard_req.limit = offset.saturating_add(limit).min(u32::MAX as usize) as u32;

    let shard_results: Vec<_> = shards
        .par_iter()
        .map(|shard| search(shard, collection_id, shard_req.clone()))
        .collect();

    let mut responses = Vec::with_capacity(shard_results.len());
    for result in shard_results {
        responses.push(result?);
    }

    Ok(merge_shard_search_responses(
        &req,
        responses,
        start.elapsed().as_micros() as u64,
        sort_value,
    ))
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub fn merge_shard_search_responses<K>(
    req: &SearchRequest,
    responses: impl IntoIterator<Item = SearchResponse>,
    took_us: u64,
    sort_value: K,
) -> SearchResponse
where
    K: Fn(&SearchHit, &str) -> Option<f64>,
{
    let offset = req.cursor.as_deref().and_then(parse_cursor).unwrap_or(0) as usize;
    let limit = req.limit as usize;
    let mut hits = Vec::new();
    let mut total = 0u64;
    for resp in responses {
        total += resp.total;
        hits.extend(resp.hits);
    }

    if let Some(sort) = &req.sort {
        hits.sort_by(|a, b| {
            for spec in sort {
                let ord = match (sort_value(a, &spec.field), sort_value(b, &spec.field)) {
                    (Some(av), Some(bv)) => av.partial_cmp(&bv).unwrap_or(Ordering::Equal),
                    _ => Ordering::Equal,
                };
                let ord = match spec.order {
                    SortOrder::Asc => ord,
                    SortOrder::Desc => ord.reverse(),
                };
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            a.external_id.cmp(&b.external_id)
        });
    } else {
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.external_id.cmp(&b.external_id))
        });
    }

    let page: Vec<_> = hits.into_iter().skip(offset).take(limit).collect();
    let next_offset = offset + page.len();
    let cursor = if (next_offset as u64) < total {
        Some(make_cursor(next_offset))
    } else {
        None
    };

    SearchResponse {
        hits: page,
        total,
        cursor,
        took_ms: took_us / 1000,
        took_us,
    }
}

fn make_cursor(offset: usize) -> String {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
    STANDARD_NO_PAD.encode(format!("{{\"offset\":{offset}}}"))
}

fn parse_cursor(s: &str) -> Option<u64> {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
    let raw = STANDARD_NO_PAD.decode(s).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&raw).ok()?;
    v.get("offset")?.as_u64()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FieldValue, QueryNode, SortMissing, SortOrder, SortSpec, TermQuery};

    #[test]
    fn shard_index_is_deterministic() {
        let a = shard_index("data-table:42", 3);
        let b = shard_index("data-table:42", 3);
        assert_eq!(a, b);
        assert!(a < 3);
    }

    #[test]
    fn shard_index_spreads() {
        let mut seen = std::collections::HashSet::new();
        for i in 0..256 {
            seen.insert(shard_index(&format!("c:{i}"), 3));
        }
        assert!(seen.len() > 1, "shard hash collapsed to a single bucket");
    }

    #[test]
    fn shard_index_single_shard_always_zero() {
        for s in ["a", "very-long-string", "中文"] {
            assert_eq!(shard_index(s, 1), 0);
        }
    }

    #[test]
    fn virtual_bucket_map_preserves_single_shard_compatibility() {
        let map = VirtualBucketShardMap::balanced(7, 128, 1).unwrap();
        for external_id in ["a", "b", "large-collection-row"] {
            let route = map.route_document("catalog", None, external_id);
            assert_eq!(route.map_version, 7);
            assert_eq!(route.virtual_bucket_count, 128);
            assert_eq!(route.physical_shard_count, 1);
            assert_eq!(route.shard, 0);
        }
    }

    #[test]
    fn virtual_bucket_map_distributes_one_large_collection_by_external_id() {
        let map = VirtualBucketShardMap::balanced(1, 1024, 8).unwrap();
        let mut seen = std::collections::BTreeSet::new();
        for i in 0..512 {
            seen.insert(
                map.route_document("one-big-collection", None, &format!("doc-{i}"))
                    .shard,
            );
        }
        assert!(
            seen.len() > 1,
            "external_id routing collapsed one collection to one shard"
        );
    }

    #[test]
    fn split_one_shard_moves_only_into_the_new_shard() {
        // 8 buckets, 2 balanced shards: shard0=[0,2,4,6], shard1=[1,3,5,7].
        let before = VirtualBucketShardMap::balanced(0, 8, 2).unwrap();
        let after = before.split_one_shard(1).unwrap();

        assert_eq!(after.version(), 1);
        assert_eq!(after.virtual_bucket_count(), 8);
        assert_eq!(after.physical_shard_count(), 3);

        let mut moved = Vec::new();
        for bucket in 0..8 {
            let old_shard = before.assignment_for_bucket(bucket).unwrap();
            let new_shard = after.assignment_for_bucket(bucket).unwrap();
            if old_shard != new_shard {
                // Every move must land on the brand-new shard, never on
                // another pre-existing shard.
                assert_eq!(new_shard, 2, "bucket {bucket} moved to an old shard");
                moved.push(bucket);
            }
        }
        // 4 buckets/shard, new_physical_shard_count=3 -> 4/3=1 bucket moves
        // from each of the 2 old shards = 2 buckets total, the lowest
        // bucket id on each source shard (0 from shard0, 1 from shard1).
        assert_eq!(moved, vec![0, 1]);
    }

    #[test]
    fn split_one_shard_is_deterministic_and_idempotent_shape() {
        let map = VirtualBucketShardMap::balanced(3, 97, 5).unwrap();
        let a = map.split_one_shard(4).unwrap();
        let b = map.split_one_shard(4).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.physical_shard_count(), 6);
        // Every bucket must still resolve to a valid shard index.
        for bucket in 0..97 {
            assert!(a.assignment_for_bucket(bucket).unwrap() < 6);
        }
    }

    #[test]
    fn split_one_shard_never_leaves_the_new_shard_empty_when_source_has_buckets() {
        let map = VirtualBucketShardMap::balanced(0, 64, 4).unwrap();
        let after = map.split_one_shard(1).unwrap();
        let new_shard_bucket_count = (0..64)
            .filter(|&b| after.assignment_for_bucket(b).unwrap() == 4)
            .count();
        assert!(
            new_shard_bucket_count > 0,
            "split produced an empty new shard"
        );
    }

    #[test]
    fn versioned_bucket_maps_can_reassign_one_bucket() {
        let key = key_for_bucket("catalog", 4, 1);
        let before = VirtualBucketShardMap::new(1, vec![0, 0, 1, 1], 2).unwrap();
        let after = VirtualBucketShardMap::new(2, vec![0, 1, 1, 1], 2).unwrap();

        let old_route = before.route_key("catalog", &key);
        let new_route = after.route_key("catalog", &key);

        assert_eq!(old_route.bucket, 1);
        assert_eq!(new_route.bucket, 1);
        assert_eq!(old_route.shard, 0);
        assert_eq!(new_route.shard, 1);
        assert_eq!(old_route.map_version, 1);
        assert_eq!(new_route.map_version, 2);
    }

    #[test]
    fn search_target_scatter_without_key_and_targets_with_key() {
        let map = VirtualBucketShardMap::balanced(3, 256, 4).unwrap();
        assert_eq!(map.search_target("catalog", None), SearchShardTarget::All);
        let SearchShardTarget::One(route) = map.search_target("catalog", Some("tenant-a")) else {
            panic!("routing key should target one shard");
        };
        assert!(route.shard < 4);
        assert_eq!(route.map_version, 3);
    }

    #[test]
    fn shard_host_formats_dns() {
        let h = shard_host("lumen", 2, "lumen-peer");
        assert_eq!(h, "lumen-2.lumen-peer");
    }

    #[test]
    fn merge_shard_search_responses_ranks_score_desc_then_external_id() {
        let req = search_req(None);
        let resp = merge_shard_search_responses(
            &req,
            [
                search_resp([hit("b", 2.0), hit("d", 1.0)], 2),
                search_resp([hit("a", 2.0), hit("c", 3.0)], 2),
            ],
            42,
            |_, _| None,
        );

        let ids: Vec<_> = resp.hits.iter().map(|h| h.external_id.as_str()).collect();
        assert_eq!(ids, ["c", "a", "b"]);
        assert_eq!(resp.total, 4);
        assert!(resp.cursor.is_some());
        assert_eq!(resp.took_us, 42);
    }

    #[test]
    fn merge_shard_search_responses_applies_global_cursor_offset() {
        let mut req = search_req(None);
        req.cursor = Some(make_cursor(2));
        let resp = merge_shard_search_responses(
            &req,
            [
                search_resp([hit("a", 4.0), hit("b", 3.0)], 2),
                search_resp([hit("c", 2.0), hit("d", 1.0)], 2),
            ],
            1000,
            |_, _| None,
        );

        let ids: Vec<_> = resp.hits.iter().map(|h| h.external_id.as_str()).collect();
        assert_eq!(ids, ["c", "d"]);
        assert_eq!(resp.cursor, None);
        assert_eq!(resp.took_ms, 1);
    }

    #[test]
    fn merge_shard_search_responses_sorts_by_resolved_number_key() {
        let mut req = search_req(None);
        req.sort = Some(vec![SortSpec {
            field: "age".into(),
            order: SortOrder::Asc,
            missing: SortMissing::Exclude,
        }]);
        let resp = merge_shard_search_responses(
            &req,
            [
                search_resp([hit("older", 1.0), hit("middle", 1.0)], 2),
                search_resp([hit("young", 1.0)], 1),
            ],
            0,
            |hit, field| match (hit.external_id.as_str(), field) {
                ("young", "age") => Some(20.0),
                ("middle", "age") => Some(35.0),
                ("older", "age") => Some(70.0),
                _ => None,
            },
        );

        let ids: Vec<_> = resp.hits.iter().map(|h| h.external_id.as_str()).collect();
        assert_eq!(ids, ["young", "middle", "older"]);
    }

    fn search_req(sort: Option<Vec<SortSpec>>) -> SearchRequest {
        SearchRequest {
            query: QueryNode::Term(TermQuery {
                field: "city".into(),
                value: FieldValue::String("taipei".into()),
            }),
            limit: 3,
            cursor: None,
            routing_key: None,
            sort,
            track_total: true,
            collapse: None,
        }
    }

    fn key_for_bucket(collection_id: &str, bucket_count: u32, desired_bucket: u32) -> String {
        for i in 0..10_000 {
            let key = format!("key-{i}");
            if route_hash(collection_id, &key) % bucket_count == desired_bucket {
                return key;
            }
        }
        panic!("could not find test key for bucket {desired_bucket}");
    }

    fn search_resp<const N: usize>(hits: [SearchHit; N], total: u64) -> SearchResponse {
        SearchResponse {
            hits: hits.into(),
            total,
            cursor: None,
            took_ms: 0,
            took_us: 0,
        }
    }

    fn hit(external_id: &str, score: f32) -> SearchHit {
        SearchHit {
            external_id: external_id.into(),
            score,
        }
    }
}
// CODEGEN-END
