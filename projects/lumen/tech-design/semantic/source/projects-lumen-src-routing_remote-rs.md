---
id: projects-lumen-src-routing-remote-rs
capability_refs:
  - id: "dynamic-shard-topology"
    role: primary
    gap: "cross-pod-shard-routing"
    claim: "cross-pod-shard-routing"
    coverage: full
    rationale: "This source unit is the sole RoutedBackend implementation: it makes operator/k8s serving pods consume the delivered shard map for cross-pod read/write routing (#1398 R1-R3), closing the gap the Surfaces caveat previously called out as separate follow-on work."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/routing_remote.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/routing_remote.rs` generated from AST during Lumen AW health remediation.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RoutedRouter` | projects/lumen/src/routing_remote.rs | struct | pub | 74 |  |
| `new` | projects/lumen/src/routing_remote.rs | function | pub | 90 | new(engine: Arc<Engine>, local_write: Arc<dyn WriteBackend>, shard_map: VirtualBucketShardMap, local_shard: u32, shard_urls: Vec<String>) -> Result<Self> |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-routing_remote-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Cross-pod shard routing for operator/k8s serving pods (#1398 R1-R3).
//!
//! [`RoutedRouter`] is the sole implementation of [`crate::api::RoutedBackend`]:
//! it consults the delivered [`crate::routing::VirtualBucketShardMap`] and
//! either answers locally (this pod owns the target bucket) or forwards to
//! the owning shard's pod over the same h2c client stack every other
//! cross-pod call in this codebase uses (`libs/h2c`, see
//! `operator::reshard_driver`'s admin forwarding for the established
//! `reqwest`-over-headless-DNS idiom this module follows). A routing-key-less
//! search scatters to every shard (local direct call + one forward per
//! remote shard) and merges through the same
//! [`crate::routing::merge_shard_search_responses`] primitive
//! [`crate::routing::EngineShardSearch`] uses.
//!
//! One-hop forwarding guard: every method checks `x-lumen-forwarded` FIRST
//! and, if present, always answers from the local engine — a forwarded
//! request never forwards again, so cross-pod routing can only ever be one
//! hop deep (R3). A forward carries the caller's `Authorization` and
//! `x-read-consistency` headers through unchanged (R3) plus
//! `x-lumen-forwarded: 1`.
//!
//! Behind the `operator` feature only because it is the sole module that
//! needs `reqwest` as a directly-nameable type — every real deployment that
//! can reach this code path already links it transitively via `operator`'s
//! `backup` feature (`dep:reqwest`), so gating here adds no new dependency
//! edge, it only keeps `reqwest::*` out of the unconditionally-compiled
//! `api.rs`.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::http::HeaderMap;
use futures::future::try_join_all;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::api::{RoutedBackend, ShardForwardRemoteError, ShardForwardUnavailable, WriteBackend};
use crate::routing::{merge_shard_search_responses, SearchShardTarget, VirtualBucketShardMap};
use crate::storage::Engine;
use crate::types::{
    IndexItem, IndexRequest, IndexResponse, ReplaceDocItem, ReplaceDocsRequest,
    ReplaceDocsResponse, SearchRequest, SearchResponse,
};

/// Internal one-hop guard header: present on every forwarded request, never
/// set by an external caller reaching this pod directly through the Service.
const FORWARDED_HEADER: &str = "x-lumen-forwarded";
/// Read-consistency header carried through a forward verbatim (R3).
const READ_CONSISTENCY_HEADER: &str = "x-read-consistency";
/// Connections per remote shard's h2c pool — small and fixed, matching
/// `operator::reshard_driver`'s admin client sizing; forwarding is bounded
/// one-hop request/response, not a bulk data-mover.
const REMOTE_POOL_CONNECTIONS: usize = 2;
const REMOTE_TIMEOUT: Duration = Duration::from_secs(10);

/// One remote shard's forwarding target: a stable headless-DNS base URL plus
/// its own small h2c connection pool. `None` at the local shard's index in
/// [`RoutedRouter::remotes`] — the local shard is never dialed.
struct RemoteShard {
    base_url: String,
    pool: h2c::H2cPool,
}

/// Routes reads and writes across physical shards for one operator/k8s
/// serving pod (#1398 R1-R3). Local-owned buckets hit `engine`/`local_write`
/// directly; remote-owned buckets forward one hop to the owning pod's
/// stable per-shard DNS name (`routing::shard_host`).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing_remote-rs.md#source
pub struct RoutedRouter {
    engine: Arc<Engine>,
    local_write: Arc<dyn WriteBackend>,
    shard_map: VirtualBucketShardMap,
    local_shard: u32,
    remotes: Vec<Option<RemoteShard>>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing_remote-rs.md#source
impl RoutedRouter {
    /// `shard_urls[shard]` is the base URL (`http://host:port`, no trailing
    /// slash) forwarded requests for that shard are sent to; its length must
    /// equal `shard_map.physical_shard_count()` and `local_shard` must be a
    /// valid index into it — both are startup-time invariants of the routed
    /// serving topology (#1398 R1), not runtime conditions, so a mismatch
    /// fails fast instead of routing into a missing shard.
    pub fn new(
        engine: Arc<Engine>,
        local_write: Arc<dyn WriteBackend>,
        shard_map: VirtualBucketShardMap,
        local_shard: u32,
        shard_urls: Vec<String>,
    ) -> Result<Self> {
        let physical = shard_map.physical_shard_count();
        if shard_urls.len() != physical as usize {
            anyhow::bail!(
                "shard_urls has {} entries but the shard map declares {physical} physical shards",
                shard_urls.len()
            );
        }
        if local_shard >= physical {
            anyhow::bail!("local_shard {local_shard} is out of range for {physical} shards");
        }
        let remotes = shard_urls
            .into_iter()
            .enumerate()
            .map(|(shard, base_url)| -> Result<Option<RemoteShard>> {
                if shard as u32 == local_shard {
                    return Ok(None);
                }
                let pool = h2c::H2cPool::with_connections_and(
                    REMOTE_POOL_CONNECTIONS,
                    Some(REMOTE_TIMEOUT),
                    Some("lumen-routed"),
                )
                .context("build routed h2c pool")?;
                Ok(Some(RemoteShard { base_url, pool }))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            engine,
            local_write,
            shard_map,
            local_shard,
            remotes,
        })
    }

    fn already_forwarded(headers: &HeaderMap) -> bool {
        headers.contains_key(FORWARDED_HEADER)
    }

    /// Sends one forwarded request and returns the raw response — success or
    /// not, decoding is the caller's job (`forward_json`/`forward_empty`)
    /// since a `DELETE` success carries no body while every other verb
    /// here returns one.
    async fn send<Req: Serialize>(
        &self,
        shard: u32,
        method: reqwest::Method,
        path: &str,
        body: Option<Req>,
        headers: &HeaderMap,
    ) -> Result<reqwest::Response> {
        let remote = self
            .remotes
            .get(shard as usize)
            .and_then(|r| r.as_ref())
            .with_context(|| format!("shard {shard} has no remote configured"))?;
        let url = format!("{}{}", remote.base_url, path);
        // Plain string header names + raw bytes, not typed `HeaderValue`s:
        // axum's and reqwest's `http` crate types aren't guaranteed to be
        // the same version, but both accept `&str`/`&[u8]` at the
        // `IntoHeaderName`/`TryInto<HeaderValue>` boundary regardless.
        let mut builder = remote
            .pool
            .client()
            .request(method.clone(), &url)
            .header(FORWARDED_HEADER, "1");
        if let Some(v) = headers.get(axum::http::header::AUTHORIZATION) {
            builder = builder.header("authorization", v.as_bytes());
        }
        if let Some(v) = headers.get(READ_CONSISTENCY_HEADER) {
            builder = builder.header(READ_CONSISTENCY_HEADER, v.as_bytes());
        }
        if let Some(b) = &body {
            builder = builder.json(b);
        }
        builder.send().await.map_err(|e| {
            anyhow::Error::new(ShardForwardUnavailable(format!("{method} {url}: {e}")))
        })
    }

    fn remote_error(status: reqwest::StatusCode, body_text: String) -> anyhow::Error {
        let message = serde_json::from_str::<service_http::ErrorEnvelope>(&body_text)
            .map(|env| format!("{}: {}", env.error, env.message))
            .unwrap_or(body_text);
        anyhow::Error::new(ShardForwardRemoteError {
            status: status.as_u16(),
            message,
        })
    }

    async fn forward_json<Req, Resp>(
        &self,
        shard: u32,
        method: reqwest::Method,
        path: &str,
        body: Option<Req>,
        headers: &HeaderMap,
    ) -> Result<Resp>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        let resp = self.send(shard, method, path, body, headers).await?;
        let status = resp.status();
        if status.is_success() {
            resp.json::<Resp>().await.map_err(|e| {
                anyhow::Error::new(ShardForwardUnavailable(format!(
                    "decode response from shard {shard} {path}: {e}"
                )))
            })
        } else {
            let body_text = resp.text().await.unwrap_or_default();
            Err(Self::remote_error(status, body_text))
        }
    }

    async fn forward_empty(
        &self,
        shard: u32,
        method: reqwest::Method,
        path: &str,
        headers: &HeaderMap,
    ) -> Result<()> {
        let resp = self.send::<()>(shard, method, path, None, headers).await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body_text = resp.text().await.unwrap_or_default();
            Err(Self::remote_error(status, body_text))
        }
    }

    /// Routing-key-less search: local engine direct + one forward per remote
    /// shard, merged through [`merge_shard_search_responses`] exactly like
    /// [`crate::routing::EngineShardSearch`]. Known gap vs. that in-process
    /// merger: `sort_value` can only resolve field values from *this* pod's
    /// local engine (`EngineShardSearch` holds every shard's `Engine`
    /// in-process and can resolve any hit; a routed pod only has its own) —
    /// a cross-shard sort-by-field page may rank remote-shard hits by a
    /// missing (`None`) sort value. Score-ranked search (the default) is
    /// unaffected.
    async fn scatter_search(
        &self,
        collection_id: &str,
        req: SearchRequest,
        headers: &HeaderMap,
    ) -> Result<SearchResponse> {
        let start = Instant::now();
        let offset = cursor_offset(req.cursor.as_deref()) as usize;
        let limit = req.limit as usize;
        let mut shard_req = req.clone();
        shard_req.cursor = None;
        shard_req.limit = offset.saturating_add(limit).min(u32::MAX as usize) as u32;

        let path = format!("/collections/{collection_id}/search");
        let mut remote_futures = Vec::new();
        for shard in 0..self.shard_map.physical_shard_count() {
            if shard == self.local_shard {
                continue;
            }
            let shard_req = shard_req.clone();
            remote_futures.push(self.forward_json::<SearchRequest, SearchResponse>(
                shard,
                reqwest::Method::POST,
                &path,
                Some(shard_req),
                headers,
            ));
        }
        let local = self.engine.search(collection_id, shard_req)?;
        let remote = try_join_all(remote_futures).await?;

        let engine = self.engine.clone();
        let collection = collection_id.to_string();
        Ok(merge_shard_search_responses(
            &req,
            std::iter::once(local).chain(remote),
            start.elapsed().as_micros() as u64,
            move |hit, field| {
                engine
                    .number_value_for_external_id(&collection, &hit.external_id, field)
                    .ok()
                    .flatten()
            },
        ))
    }
}

/// Duplicates `routing::parse_cursor`'s tiny base64+JSON offset decode
/// (private to that module) rather than widening its visibility for one
/// caller — this is incidental cursor-codec plumbing, not the reusable
/// merge primitive (`merge_shard_search_responses`) this module already
/// shares with `routing.rs`.
fn cursor_offset(cursor: Option<&str>) -> u64 {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
    let Some(s) = cursor else {
        return 0;
    };
    let Ok(raw) = STANDARD_NO_PAD.decode(s) else {
        return 0;
    };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&raw) else {
        return 0;
    };
    v.get("offset").and_then(|o| o.as_u64()).unwrap_or(0)
}

#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-routing_remote-rs.md#source
impl RoutedBackend for RoutedRouter {
    async fn search(
        &self,
        collection_id: &str,
        req: SearchRequest,
        headers: &HeaderMap,
    ) -> Result<SearchResponse> {
        if Self::already_forwarded(headers) {
            return self.engine.search(collection_id, req);
        }
        match self
            .shard_map
            .search_target(collection_id, req.routing_key.as_deref())
        {
            SearchShardTarget::One(route) if route.shard == self.local_shard => {
                self.engine.search(collection_id, req)
            }
            SearchShardTarget::One(route) => {
                let path = format!("/collections/{collection_id}/search");
                self.forward_json(
                    route.shard,
                    reqwest::Method::POST,
                    &path,
                    Some(req),
                    headers,
                )
                .await
            }
            SearchShardTarget::All => self.scatter_search(collection_id, req, headers).await,
        }
    }

    async fn index(
        &self,
        collection_id: String,
        req: IndexRequest,
        headers: &HeaderMap,
    ) -> Result<IndexResponse> {
        if Self::already_forwarded(headers) {
            return self.local_write.index(collection_id, req).await;
        }
        let shard_count = self.shard_map.physical_shard_count() as usize;
        let mut shard_items: Vec<Vec<IndexItem>> = (0..shard_count).map(|_| Vec::new()).collect();
        for item in req.items {
            let shard = self
                .shard_map
                .route_document(&collection_id, None, &item.external_id)
                .shard as usize;
            shard_items[shard].push(item);
        }

        let path = format!("/collections/{collection_id}/index");
        let mut local_resp: Option<IndexResponse> = None;
        let mut remote_futures = Vec::new();
        for (shard, items) in shard_items.into_iter().enumerate() {
            if items.is_empty() {
                continue;
            }
            let shard = shard as u32;
            let sub_req = IndexRequest {
                items,
                request_id: req.request_id.clone(),
            };
            if shard == self.local_shard {
                local_resp = Some(
                    self.local_write
                        .index(collection_id.clone(), sub_req)
                        .await?,
                );
            } else {
                remote_futures.push(self.forward_json::<IndexRequest, IndexResponse>(
                    shard,
                    reqwest::Method::POST,
                    &path,
                    Some(sub_req),
                    headers,
                ));
            }
        }
        let remote_resps = try_join_all(remote_futures).await?;

        let mut indexed = 0u32;
        let mut bytes_written = BTreeMap::new();
        let mut shard_lag_ms = 0u64;
        for resp in local_resp.into_iter().chain(remote_resps) {
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
        headers: &HeaderMap,
    ) -> Result<ReplaceDocsResponse> {
        if Self::already_forwarded(headers) {
            return self.local_write.replace_docs(collection_id, req).await;
        }
        let total = req.docs.len();
        let shard_count = self.shard_map.physical_shard_count() as usize;
        let mut shard_docs: Vec<Vec<ReplaceDocItem>> =
            (0..shard_count).map(|_| Vec::new()).collect();
        let mut shard_positions: Vec<Vec<usize>> = (0..shard_count).map(|_| Vec::new()).collect();
        for (idx, item) in req.docs.into_iter().enumerate() {
            let shard = self
                .shard_map
                .route_document(&collection_id, None, &item.external_id)
                .shard as usize;
            shard_positions[shard].push(idx);
            shard_docs[shard].push(item);
        }

        let path = format!("/collections/{collection_id}/docs:replace");
        let mut local_resp: Option<(u32, ReplaceDocsResponse)> = None;
        let mut remote_shards = Vec::new();
        let mut remote_futures = Vec::new();
        for (shard, docs) in shard_docs.into_iter().enumerate() {
            if docs.is_empty() {
                continue;
            }
            let shard = shard as u32;
            let sub_req = ReplaceDocsRequest { docs };
            if shard == self.local_shard {
                local_resp = Some((
                    shard,
                    self.local_write
                        .replace_docs(collection_id.clone(), sub_req)
                        .await?,
                ));
            } else {
                remote_shards.push(shard);
                remote_futures.push(
                    self.forward_json::<ReplaceDocsRequest, ReplaceDocsResponse>(
                        shard,
                        reqwest::Method::PUT,
                        &path,
                        Some(sub_req),
                        headers,
                    ),
                );
            }
        }
        let remote_resps = try_join_all(remote_futures).await?;

        let mut results: Vec<Option<crate::types::ReplaceDocResult>> =
            (0..total).map(|_| None).collect();
        if let Some((shard, resp)) = local_resp {
            for (pos, r) in shard_positions[shard as usize].iter().zip(resp.results) {
                results[*pos] = Some(r);
            }
        }
        for (shard, resp) in remote_shards.into_iter().zip(remote_resps) {
            for (pos, r) in shard_positions[shard as usize].iter().zip(resp.results) {
                results[*pos] = Some(r);
            }
        }
        let results = results
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
        headers: &HeaderMap,
    ) -> Result<()> {
        if Self::already_forwarded(headers) {
            return self
                .local_write
                .delete(collection_id, external_id, field)
                .await;
        }
        let route = self
            .shard_map
            .route_document(&collection_id, None, &external_id);
        if route.shard == self.local_shard {
            return self
                .local_write
                .delete(collection_id, external_id, field)
                .await;
        }
        let mut path = format!("/collections/{collection_id}/index/{external_id}");
        if let Some(f) = &field {
            path.push('?');
            path.push_str("field=");
            path.push_str(f);
        }
        self.forward_empty(route.shard, reqwest::Method::DELETE, &path, headers)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::DropOutcome;
    use crate::types::{CreateCollectionRequest, CreateCollectionResponse};
    use async_trait::async_trait;

    struct DummyWrite;

    #[async_trait]
    impl WriteBackend for DummyWrite {
        async fn create_collection(
            &self,
            _collection_id: String,
            _req: CreateCollectionRequest,
        ) -> Result<CreateCollectionResponse> {
            unimplemented!("construction-only test double")
        }
        async fn drop_collection(
            &self,
            _collection_id: String,
            _force: bool,
        ) -> Result<DropOutcome> {
            unimplemented!("construction-only test double")
        }
        async fn index(&self, _collection_id: String, _req: IndexRequest) -> Result<IndexResponse> {
            unimplemented!("construction-only test double")
        }
        async fn replace_docs(
            &self,
            _collection_id: String,
            _req: ReplaceDocsRequest,
        ) -> Result<ReplaceDocsResponse> {
            unimplemented!("construction-only test double")
        }
        async fn delete(
            &self,
            _collection_id: String,
            _external_id: String,
            _field: Option<String>,
        ) -> Result<()> {
            unimplemented!("construction-only test double")
        }
        async fn drop_field(&self, _collection_id: String, _field_name: String) -> Result<u32> {
            unimplemented!("construction-only test double")
        }
    }

    fn shard_map(physical: u32) -> VirtualBucketShardMap {
        VirtualBucketShardMap::balanced(0, 16, physical).unwrap()
    }

    #[test]
    fn new_rejects_mismatched_shard_url_count() {
        let err = RoutedRouter::new(
            Arc::new(Engine::new()),
            Arc::new(DummyWrite),
            shard_map(3),
            0,
            vec!["http://a".into(), "http://b".into()],
        )
        .err()
        .unwrap();
        assert!(err.to_string().contains('3'));
    }

    #[test]
    fn new_rejects_out_of_range_local_shard() {
        let err = RoutedRouter::new(
            Arc::new(Engine::new()),
            Arc::new(DummyWrite),
            shard_map(2),
            2,
            vec!["http://a".into(), "http://b".into()],
        )
        .err()
        .unwrap();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn new_accepts_valid_topology() {
        let router = RoutedRouter::new(
            Arc::new(Engine::new()),
            Arc::new(DummyWrite),
            shard_map(2),
            0,
            vec![
                "http://search-0.headless".into(),
                "http://search-1.headless".into(),
            ],
        )
        .unwrap();
        assert!(
            router.remotes[0].is_none(),
            "local shard has no remote entry"
        );
        assert!(router.remotes[1].is_some());
    }

    #[test]
    fn already_forwarded_detects_marker_header() {
        let mut headers = HeaderMap::new();
        assert!(!RoutedRouter::already_forwarded(&headers));
        headers.insert(FORWARDED_HEADER, "1".parse().unwrap());
        assert!(RoutedRouter::already_forwarded(&headers));
    }

    #[test]
    fn cursor_offset_decodes_offset_and_defaults_to_zero() {
        assert_eq!(cursor_offset(None), 0);
        assert_eq!(cursor_offset(Some("not-base64!!")), 0);

        use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
        let cursor = STANDARD_NO_PAD.encode(r#"{"offset":42}"#);
        assert_eq!(cursor_offset(Some(&cursor)), 42);
    }
}
// CODEGEN-END
````

```yaml
changes:
  - path: "projects/lumen/src/routing_remote.rs"
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      New module (#1398 R1-R3): `RoutedRouter`, the sole implementation of
      `crate::api::RoutedBackend`. Local-owned virtual buckets are answered
      directly from `engine`/`local_write`; remote-owned buckets forward one
      hop over `libs/h2c` to the owning shard pod's stable headless-DNS name
      (`routing::shard_host`), following the same `reqwest`-over-h2c idiom
      already established by `operator::reshard_driver`'s admin forwarding.
      Every `RoutedBackend` method checks the `x-lumen-forwarded` header
      first and always answers locally when it is present, bounding
      cross-pod forwarding to exactly one hop (R3, no forwarding loops). A
      forward carries the caller's `Authorization` and `x-read-consistency`
      headers through unchanged. Routing-key-less search
      (`scatter_search`) fans out to every shard and merges through the
      same `routing::merge_shard_search_responses` primitive
      `routing::EngineShardSearch` uses in the non-k8s fan-in mode.
```
