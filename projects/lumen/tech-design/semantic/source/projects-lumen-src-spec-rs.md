---
id: projects-lumen-src-spec-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/spec.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `field_catalog` | projects/lumen/src/spec.rs | function | pub | 110 | field_catalog() -> Value |
| `json_schema_json` | projects/lumen/src/spec.rs | function | pub | 30 | json_schema_json() -> String |
| `llm_integration_md` | projects/lumen/src/spec.rs | function | pub | 215 | llm_integration_md() -> String |
| `llm_outline_md` | projects/lumen/src/spec.rs | function | pub | 131 | llm_outline_md() -> String |
| `llm_quickstart_md` | projects/lumen/src/spec.rs | function | pub | 250 | llm_quickstart_md() -> String |
| `llm_recipes_md` | projects/lumen/src/spec.rs | function | pub | 306 | llm_recipes_md() -> String |
| `llm_workflow_md` | projects/lumen/src/spec.rs | function | pub | 155 | llm_workflow_md() -> String |
| `openapi_json` | projects/lumen/src/spec.rs | function | pub | 15 | openapi_json() -> String |
| `openapi_yaml` | projects/lumen/src/spec.rs | function | pub | 23 | openapi_yaml() -> String |
| `query_shapes` | projects/lumen/src/spec.rs | function | pub | 41 | query_shapes() -> Value |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Offline, machine-readable self-description for agent integration.
//!
//! The `lumen spec` CLI subset emits everything an LLM agent needs to wire
//! lumen into a RAG / tool pipeline — schema, query-shape cookbook, field /
//! analyzer catalog — straight from the installed binary, with no running
//! server and no network. This module is the single source for that surface;
//! the CLI and the (legacy) `lumen-openapi-dump` binary both call into it.

use serde_json::{json, Value};

/// The full OpenAPI 3 document as pretty JSON (every route + schema).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn openapi_json() -> String {
    crate::api::openapi()
        .to_pretty_json()
        .expect("OpenApi serializes to JSON")
}

/// The full OpenAPI 3 document as YAML for LLM/agent reading.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn openapi_yaml() -> String {
    serde_yaml::to_string(&crate::api::openapi()).expect("OpenApi serializes to YAML")
}

/// Just the component schemas (the request/response data types) as pretty JSON
/// — the JSON-Schema view an agent uses to build/validate request bodies.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn json_schema_json() -> String {
    let api = crate::api::openapi();
    serde_json::to_string_pretty(&json!({ "components": api.components }))
        .expect("components serialize to JSON")
}

/// A cookbook of canonical query shapes. Each entry is a ready-to-POST
/// `{name, description, request}` for `POST /collections/{id}/search` (or
/// `/duplicates` where noted) using the exact wire form of every `QueryNode`
/// variant plus sort / collapse.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn query_shapes() -> Value {
    json!({
        "search_endpoint": "POST /collections/{collection}/search",
        "note": "lumen returns ranked/sorted external_id hits only — never documents.",
        "shapes": [
            { "name": "term", "description": "exact keyword/number/bool match",
              "request": { "query": { "term": { "field": "status", "value": "active" } }, "limit": 20 } },
            { "name": "terms", "description": "keyword in a set (IN)",
              "request": { "query": { "terms": { "field": "status", "values": ["active", "trial"] } }, "limit": 20 } },
            { "name": "ids", "description": "filter by a set of external_ids (row_id_in); unknown ids skipped",
              "request": { "query": { "ids": { "values": ["row-42", "row-91"] } }, "limit": 20 } },
            { "name": "range", "description": "numeric range (e.g. 1000 <= price < 5000)",
              "request": { "query": { "range": { "field": "price", "gte": 1000, "lt": 5000 } }, "limit": 20 } },
            { "name": "match_bm25", "description": "lexical BM25 ranking over a text field",
              "request": { "query": { "match": { "field": "bio", "text": "rust search engineer" } }, "limit": 20 } },
            { "name": "boolean_and", "description": "conjunction; planner drives from the most selective clause",
              "request": { "query": { "and": [
                  { "match": { "field": "name", "text": "手機殼" } },
                  { "range": { "field": "price", "gte": 1000, "lt": 5000 } }
              ] }, "limit": 20 } },
            { "name": "boolean_or", "description": "disjunction",
              "request": { "query": { "or": [
                  { "term": { "field": "brand", "value": "apple" } },
                  { "term": { "field": "brand", "value": "samsung" } }
              ] }, "limit": 20 } },
            { "name": "boolean_not", "description": "AND with a negated filter clause",
              "request": { "query": { "and": [
                  { "term": { "field": "category", "value": "phone" } },
                  { "not": { "term": { "field": "refurbished", "value": "true" } } }
              ] }, "limit": 20 } },
            { "name": "knn", "description": "vector kNN (caller supplies the embedding)",
              "request": { "query": { "knn": { "field": "embedding", "vector": [0.12, -0.03, 0.88], "k": 10 } }, "limit": 10 } },
            { "name": "rrf_hybrid", "description": "hybrid lexical+semantic: fuse a BM25 match and a vector kNN by rank (Reciprocal Rank Fusion)",
              "request": { "query": { "rrf": { "k": 60, "queries": [
                  { "match": { "field": "title", "text": "wireless earbuds" } },
                  { "knn": { "field": "embedding", "vector": [0.12, -0.03, 0.88], "k": 50 } }
              ] } }, "limit": 10 } },
            { "name": "rrf_hybrid_filtered", "description": "filter-correct hybrid: put the filter INSIDE each leg so the kNN leg stays filter-correct (no recall collapse)",
              "request": { "query": { "rrf": { "k": 60, "queries": [
                  { "and": [ { "match": { "field": "title", "text": "wireless earbuds" } }, { "term": { "field": "brand", "value": "acme" } } ] },
                  { "and": [ { "knn": { "field": "embedding", "vector": [0.12, -0.03, 0.88], "k": 50 } }, { "term": { "field": "brand", "value": "acme" } } ] }
              ] } }, "limit": 10 } },
            { "name": "hamming_near_dup", "description": "perceptual near-duplicate: hashes within N Hamming bits",
              "request": { "query": { "hamming": { "field": "phash", "hash": "f0e1d2c3b4a59687", "max_distance": 8 } }, "limit": 20 } },
            { "name": "has_child_nested_group", "description": "rows whose nested group has an element matching a sub-query; may be combined with parent-field sort",
              "request": { "query": { "has_child": {
                  "collection": "orders_items", "field": "parent_row_id",
                  "query": { "and": [
                      { "term": { "field": "sku", "value": "S0" } },
                      { "range": { "field": "qty", "gte": 5 } }
                  ] } } },
                  "sort": [ { "field": "score", "order": "asc" } ],
                  "track_total": true,
                  "limit": 20 } },
            { "name": "collapse_group_by", "description": "one hit per distinct keyword value (group-by), scored by the max member",
              "request": { "query": { "term": { "field": "in_stock", "value": "true" } }, "collapse": "brand", "limit": 20 } },
            { "name": "filter_then_sort", "description": "filter, then sort by a field instead of relevance",
              "request": { "query": { "range": { "field": "price", "gte": 100 } },
                           "sort": [ { "field": "price", "order": "asc" } ], "track_total": false, "limit": 20 } },
            { "name": "duplicates", "description": "find external_ids sharing a value (POST /collections/{id}/duplicates)",
              "request": { "field": "email", "min_group_size": 2, "limit": 100 } }
        ]
    })
}

/// The field-type + analyzer + vector-metric catalog — what `type`/`analyzer`/
/// `metric` values a `PUT /collections/{id}` schema may use. Mirrors the
/// `FieldType` / `Analyzer` / `VectorMetric` enums.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn field_catalog() -> Value {
    json!({
        "schema_endpoint": "PUT /collections/{collection}",
        "field_types": [
            { "type": "text", "purpose": "BM25 lexical ranking; tokenized at index time", "analyzers": ["whitespace_lower", "ngram", "jieba"] },
            { "type": "keyword", "purpose": "exact term / set membership / enum path; roaring postings" },
            { "type": "number", "purpose": "numeric range + sort (dates as epoch)" },
            { "type": "set", "purpose": "multi-valued keyword membership" },
            { "type": "vector", "purpose": "semantic kNN over a caller-supplied embedding (HNSW)", "metrics": ["cosine", "dot", "l2"] },
            { "type": "hash", "purpose": "perceptual/structural near-dup search — 64-bit hex hash, queried by Hamming distance (pHash / SimHash / b-bit MinHash)" }
        ],
        "analyzers": [
            { "name": "whitespace_lower", "purpose": "split on whitespace, lowercase (default lexical)" },
            { "name": "ngram", "purpose": "character n-grams — substring and CJK matching" },
            { "name": "jieba", "purpose": "Chinese word segmentation (requires the `jieba` build feature)" }
        ]
    })
}

/// The agent-facing LLM topic outline (`lumen llm outline`) as Markdown.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn llm_outline_md() -> String {
    r#"# lumen LLM outline

Use the smallest topic that answers the task:

- `lumen llm workflow` — product model, declare→ingest→search→hydrate, query
  flavor choices, connection, and non-goals.
- `lumen llm integration` — recommended Postgres/AlloyDB adapter boundary:
  outbox or CDC, external Pub/Sub retry/DLQ ownership, HTTP writes into lumen,
  and no direct external writes to lumen's internal broker WAL.
- `lumen llm quickstart` — copy-paste local create → index → search flow.
- `lumen llm recipes` — task → ready-to-POST query bodies.
- `lumen spec --format openapi-yaml` — OpenAPI YAML for LLM/agent reading.
- `lumen spec` — OpenAPI JSON, JSON-schema, query-shape, field, analyzer, and
  vector metric catalogs.
"#
    .to_string()
}

/// The agent workflow model (`lumen llm workflow`) as Markdown — the mental
/// model, declare→ingest→search→hydrate workflow, search-flavor decision map,
/// connection, and non-goals. Where exact wire shape is needed it points at
/// `lumen spec` / `lumen llm recipes` so there is one source of truth.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn llm_workflow_md() -> String {
    r#"# lumen workflow

## What lumen is
lumen is a **search index, not a database**. You (the caller) own the source of
truth — Postgres / AlloyDB / MongoDB / S3. lumen stores only index bits keyed by
your `external_id` and returns **ranked `external_id`s, never documents**. You
hydrate the hits against your own store.

## The integration loop (4 steps)
1. **Declare** a collection schema once — `PUT /collections/{id}` with a map of
   field name → typed field. The type fixes the index; there is no separate
   "index options" knob.
2. **Ingest** — your own pub/sub (CDC / logical replication / app writes) calls
   `POST /collections/{id}/index`. lumen bundles no connector; see
   `examples/consumer_pg_logical.py`. Re-writing `(external_id, field)` fully
   re-indexes that field.
3. **Search** — `POST /collections/{id}/search` with a query (relevance +
   filters + sort). You get back ranked `external_id`s + scores.
4. **Hydrate** — look the returned `external_id`s up in YOUR store to get the
   full records. lumen never had them.

## Which "find" to use
- exact value / membership → `keyword` (`term`, `terms`) or `set`
- numeric / date range → `number` (`range`)
- full-text relevance → `text` + `match` (BM25). Analyzers: `whitespace_lower`,
  `ngram` (substring/CJK), `jieba` (Chinese)
- semantic similarity → `vector` + `knn` (you supply the embedding)
- perceptual / near-duplicate → `hash` + `hamming`
- hybrid lexical+semantic → `rrf` (fuse `match` + `knn` by rank; put any filter
  INSIDE each leg so the kNN leg stays filter-correct)
- which `external_id`s share a value → `POST /duplicates`
- nested data-table / "parent whose child matches" → `has_child`; combine it
  with parent-field `sort` for list-row flows that filter by child rows then
  order/count parent rows
- compose any of the above under `and` / `or` / `not`

## Connection
HTTP/1.1 or HTTP/2 cleartext on `:7373` — any REST client, no driver. When the
node runs with `LUMEN_AUTH=required`, send `Authorization: Bearer <token>`.
Sharded deployments route on the client: `crc32(collection_id) % shard_count`.

## Do NOT ask lumen to
- store or return documents — it returns `external_id`s; hydrate them yourself
- run transactions or be the system of record
- aggregate (group-by / histogram / percentile / cardinality) — pair it with an
  OLAP store (ClickHouse / Druid / BigQuery / DuckDB)
- generate embeddings or hashes — you compute them; lumen indexes the bits

## Exact wire shapes
`lumen spec` (OpenAPI), `lumen spec --shapes` (query cookbook), `lumen spec
--fields` (field/analyzer catalog), or `lumen llm recipes` (task → ready-to-POST
body). `lumen llm integration` covers database/pubsub adapter boundaries.
"#
    .to_string()
}

/// The recommended database/pubsub integration boundary (`lumen llm
/// integration`) as Markdown.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn llm_integration_md() -> String {
    r#"# lumen integration

## Recommended Postgres / AlloyDB integration
Use this boundary when Postgres or AlloyDB is the source of truth:
1. Commit application data in the database first. If you need crash-safe
   delivery, write an outbox row in the same transaction or consume CDC from
   the committed log; do not make lumen a transaction participant.
2. Run an adapter/sidecar that consumes CDC, Pub/Sub, Kafka, or the outbox and
   translates each source change into lumen HTTP writes (`POST
   /collections/{id}/index` and the delete endpoint). The adapter owns cloud
   envelopes, ACK/retry/DLQ policy, upstream auth, and stale-event filtering.
3. POST to the collection's shard and ACK upstream only after lumen returns
   success. Replaying an upsert of `(external_id, field)` is safe because it
   replaces that field; retry deletes until they succeed.
4. If upstream delivery can arrive out of order, carry a monotonic
   `source_version` / commit LSN in the adapter and suppress stale writes before
   POSTing.
5. Do not publish directly to lumen's broker stream. Relay is lumen's internal
   WAL and fan-out substrate; external producers use the HTTP API so
   every write goes through validation, routing, and the same log/apply path.

## Ownership boundary
- lumen core owns schema validation, sharded HTTP writes, the internal WAL,
  ordered apply, search, and ranked `external_id` responses.
- The adapter owns source-specific envelopes, Pub/Sub subscription settings,
  ACK/retry/DLQ, upstream credentials, source offsets, stale-event suppression,
  and hydration against the source database.
"#
    .to_string()
}

/// A copy-paste end-to-end (`lumen llm quickstart`) as Markdown: create → index
/// → search against a local `lumen serve` on `:7373`.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn llm_quickstart_md() -> String {
    r#"# lumen quickstart (copy-paste)

Assumes a node at `http://localhost:7373` (`lumen serve`). Add
`-H 'authorization: Bearer <token>'` when `LUMEN_AUTH=required`.

## 1. Declare a collection
```bash
curl -sS -XPUT localhost:7373/collections/products \
  -H 'content-type: application/json' -d '{
    "fields": {
      "title":     { "type": "text", "analyzer": "whitespace_lower" },
      "brand":     { "type": "keyword" },
      "price":     { "type": "number" },
      "embedding": { "type": "vector", "dim": 3, "metric": "cosine" }
    }
  }'
```

## 2. Index items (your pub/sub does this in production)
```bash
curl -sS -XPOST localhost:7373/collections/products/index \
  -H 'content-type: application/json' -d '{
    "items": [
      { "external_id": "p1", "field": "title", "value": "wireless earbuds" },
      { "external_id": "p1", "field": "brand", "value": "acme" },
      { "external_id": "p1", "field": "price", "value": 79 },
      { "external_id": "p1", "field": "embedding", "value": [0.1, 0.2, 0.9] }
    ]
  }'
```

## 3. Search (filters + relevance)
```bash
curl -sS -XPOST localhost:7373/collections/products/search \
  -H 'content-type: application/json' -d '{
    "query": { "and": [
      { "match": { "field": "title", "text": "earbuds" } },
      { "range": { "field": "price", "lte": 100 } }
    ] },
    "limit": 10
  }'
```

## 4. Hydrate
The response is `{ "hits": [ { "external_id", "score" } ], ... }`. Fetch the full
records from YOUR store by those `external_id`s — lumen never stored them.

More shapes: `lumen llm recipes`. Full schema: `lumen spec`.
"#
    .to_string()
}

/// Task → ready-to-POST body recipes (`lumen llm recipes`) as Markdown, rendered
/// from [`query_shapes`] so the bodies never drift from the canonical cookbook.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md#source
pub fn llm_recipes_md() -> String {
    let shapes = query_shapes();
    let endpoint = shapes["search_endpoint"].as_str().unwrap_or("");
    let mut out = String::from("# lumen query recipes\n\n");
    if !endpoint.is_empty() {
        out.push_str(&format!("Search endpoint: `{endpoint}`\n\n"));
    }
    out.push_str(
        "Each recipe is a ready-to-POST request body. Same source as `lumen spec \
         --shapes`.\n\n",
    );
    if let Some(list) = shapes["shapes"].as_array() {
        for s in list {
            let name = s["name"].as_str().unwrap_or("recipe");
            let desc = s["description"].as_str().unwrap_or("");
            let req = serde_json::to_string_pretty(&s["request"]).unwrap_or_default();
            out.push_str(&format!("## {name}\n{desc}\n\n```json\n{req}\n```\n\n"));
        }
    }
    out
}
// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/spec.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/spec.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
