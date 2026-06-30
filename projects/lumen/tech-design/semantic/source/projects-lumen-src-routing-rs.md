---
id: projects-lumen-src-routing-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/routing.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/routing.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `EngineShardSearch` | projects/lumen/src/routing.rs | struct | pub | 63 |  |
| `EngineShardWrite` | projects/lumen/src/routing.rs | struct | pub | 106 |  |
| `document_shard_index` | projects/lumen/src/routing.rs | function | pub | 45 | document_shard_index(collection_id: &str, external_id: &str, shard_count: usize) -> usize |
| `is_empty` | projects/lumen/src/routing.rs | function | pub | 79 | is_empty(&self) -> bool |
| `is_empty` | projects/lumen/src/routing.rs | function | pub | 122 | is_empty(&self) -> bool |
| `len` | projects/lumen/src/routing.rs | function | pub | 75 | len(&self) -> usize |
| `len` | projects/lumen/src/routing.rs | function | pub | 118 | len(&self) -> usize |
| `merge_shard_search_responses` | projects/lumen/src/routing.rs | function | pub | 363 | merge_shard_search_responses(     req: &SearchRequest,     responses: impl IntoIterator<Item = SearchResponse>,     took_us: u64,     sort_value: K, ) -> SearchResponse |
| `new` | projects/lumen/src/routing.rs | function | pub | 69 | new(shards: Vec<Arc<Engine>>) -> Self |
| `new` | projects/lumen/src/routing.rs | function | pub | 112 | new(writers: Vec<Arc<WriteCoordinator>>) -> Self |
| `search_shards_parallel` | projects/lumen/src/routing.rs | function | pub | 325 | search_shards_parallel(     collection_id: &str,     req: SearchRequest,     shards: &[S],     search: F,     sort_value: K, ) -> Result<SearchResponse> |
| `shard_host` | projects/lumen/src/routing.rs | function | pub | 57 | shard_host(prefix: &str, shard: u32, headless_service: &str) -> String |
| `shard_index` | projects/lumen/src/routing.rs | function | pub | 32 | shard_index(collection_id: &str, shard_count: u32) -> u32 |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Shard routing.
//!
//! Two layers, split between client and server:
//!   Layer 1 (client) — shard math: `crc32(collection_id) % shard_count`.
//!   Layer 2 (server) — Raft leader forwarding inside the shard.
//!
//! Clients only need Layer 1; the server handles re-election transparently.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::time::Instant;

use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;
use futures::future::try_join_all;
use rayon::prelude::*;

use crate::api::{SearchBackend, WriteBackend};
use crate::coordinator::WriteCoordinator;
use crate::log_entry::RaftLogEntry;
use crate::storage::{ApplyOutcome, DropOutcome, Engine};
use crate::types::{
    CreateCollectionRequest, CreateCollectionResponse, IndexRequest, IndexResponse, SearchHit,
    SearchRequest, SearchResponse, SortOrder,
};

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub fn shard_index(collection_id: &str, shard_count: u32) -> u32 {
    debug_assert!(shard_count > 0, "shard_count must be > 0");
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(collection_id.as_bytes());
    hasher.finalize() % shard_count
}

/// Row/document routing for a sharded local serving node. Collection-level
/// routing (`shard_index`) sends a whole collection to a cluster shard; this
/// function splits documents *inside* that collection across local shard engines
/// so write apply can run on multiple cores while each document remains owned by
/// exactly one shard.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
pub fn document_shard_index(collection_id: &str, external_id: &str, shard_count: usize) -> usize {
    debug_assert!(shard_count > 0, "shard_count must be > 0");
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(collection_id.as_bytes());
    hasher.update(&[0]);
    hasher.update(external_id.as_bytes());
    (hasher.finalize() as usize) % shard_count
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
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
impl EngineShardSearch {
    pub fn new(shards: Vec<Arc<Engine>>) -> Self {
        Self {
            shards: Arc::new(shards),
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
        search_shards_parallel(
            collection_id,
            req,
            self.shards.as_slice(),
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
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing-rs.md#source
impl EngineShardWrite {
    pub fn new(writers: Vec<Arc<WriteCoordinator>>) -> Self {
        Self {
            writers: Arc::new(writers),
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
            let shard = document_shard_index(&collection_id, &item.external_id, self.writers.len());
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

    async fn delete(
        &self,
        collection_id: String,
        external_id: String,
        field: Option<String>,
    ) -> Result<()> {
        self.require_shards()?;
        let shard = document_shard_index(&collection_id, &external_id, self.writers.len());
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
            sort,
            track_total: true,
            collapse: None,
        }
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

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/routing.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/routing.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
