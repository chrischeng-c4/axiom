# Lumen querying

## Purpose

This guide defines how Lumen selects, scores, orders, and summarizes indexed
IDs. It also defines the boundary between search and source-record hydration.

Current behavior and the Search v2 target are separate. The Search v2 query, result,
facet, metric, limit, cache, and admission contracts in this guide are future
work. Use the [support matrix](../STATUS.md) for current support.

## Data ownership

PostgreSQL, another database, or an object store remains the source of truth.
Lumen stores a derived index and returns ordered source IDs plus search
metadata. It does not return source records.

The caller owns:

- CDC or outbox checkpoints, source-delivery retry, and freshness;
- the mapping from source data to indexed fields;
- execution of any embedding model and production of raw vectors or
  perceptual hashes;
- loading source records by an ID list;
- restoring the order returned by Lumen after that load; and
- any source-record authorization, projection, and final hydration.

Lumen owns business filter, scoring, sort, limit, and cursor execution over the
derived index. A normal request path is:

1. The caller writes source data to its source of truth.
2. The caller projects searchable values into Lumen.
3. Lumen returns ordered `external_id` values and search metadata.
4. The caller loads source records with one bulk ID-list request.
5. The caller or the planned generated-client helper restores Lumen's order and
   returns its application response.

The source database should not repeat Lumen's business `WHERE`, `ORDER BY`, or
`LIMIT` work. It should load the selected IDs. General aggregation remains a
separate source-database or analytics concern.

Lumen keeps raw vector, kNN, RRF, perceptual-hash, and Hamming search. It does
not execute an embedding model. The planned generated client owns one-request
mechanics such as typed errors, safe retry, deadline, cancellation, and ordered
hydration helpers. The caller still owns source access and freshness.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| Current HTTP methods and wire schemas | [`lumen spec` OpenAPI](../clients/openapi.json) | Run `lumen spec` or request `GET /openapi.json`. |
| Source ownership, selection, and hydration | [Data ownership](#data-ownership) | Use this flow when integrating a source database. |
| Current query nodes and controls | Current OpenAPI and `lumen spec --shapes` | Run `lumen spec --shapes` for current request examples. |
| Search v2 query, scoring, and filter semantics | [Search model](#search-model) | Run `lumen llm --topic querying --format json` for current-versus-target navigation. |
| Search v2 ordering, pagination, totals, and collapse | [Result controls](#result-controls) | Use the versioned result contract before changing a caller. |
| Search v2 facets, metrics, precision, and wire names | [Facets and metrics](#facets-and-metrics) | Treat these definitions as target behavior until STATUS changes. |
| Search v2 complexity, failure, admission, and cache rules | [Limits and failures](#limits-and-failures) | Handle the exact error status and `Retry-After` rules. |
| Current contract to Search v2 caller changes | [Search v2 migration](migration-search-v2.md) | Use the compatibility table and future offline migration tools. |
| Current implementation support | [STATUS.md](../STATUS.md) | Read each query support row and its evidence. |
| Future completion evidence | [ROADMAP.md](../ROADMAP.md) | Follow the outcome linked from a current limit. |

## Search model

### Current model

The current SearchRequest has one combined query tree. Nodes such as `term`,
`range`, `match`, `knn`, `rrf`, `and`, `or`, and `not` can participate in that
tree. The current wire model does not provide the Search v2 typed separation between
scoring and filtering.

Use `lumen spec --shapes` and current OpenAPI for the implemented nodes. Do not
send the new Search v2 fields to a mixed-version runtime.

### Search v2 selection and scoring

A Search v2 request has two optional top-level inputs:

- `query: ScoringQuery` selects and scores candidates.
- `filter: FilterExpr` selects candidates without changing score.

When both inputs are absent, the request is match-all. When both are present,
Lumen returns their intersection. A filter never changes score. A filter-only
hit has `score: null`.

A scoring query keeps its score when the final result uses a field sort.
Filters run before kNN candidate selection. They also run before candidate
selection for every RRF leg.

`ScoringQuery` supports:

- lexical match;
- phrase;
- fuzzy;
- kNN;
- Reciprocal Rank Fusion, or RRF;
- Hamming search; and
- lexical bool composition.

`FilterExpr` supports:

- `term`, `terms`, `range`, `exists`, `ids`, and `prefix`;
- `and`, `or`, and `not`;
- `has_child`; and
- `duplicated`.

For lexical bool, `minimum_should_match` defaults to `1` when there is no
`must` clause. It defaults to `0` when a `must` clause exists. A caller can
provide another valid explicit value.

kNN cannot appear inside lexical bool. Use RRF for a heterogeneous hybrid such
as lexical plus vector search.

`has_child` is a bounded semi-join. The child query decides whether a matching
child exists. A child score never becomes the parent score.

Cross-collection search returns independent result sets. It does not perform a
join, merged ranking, or merged cursor across collections.

See [Unified search contract](../ROADMAP.md#unified-search-contract). None of
the typed Search v2 separation is supported today.

## Result controls

### Current controls

The current API supports `limit`, `cursor`, `offset`, `sort`, boolean
`track_total`, and current keyword collapse as declared by OpenAPI. The
[README](../README.md#primary-workflow) shows the current response form.

The result contract below replaces several current shapes in Search v2. Use the
[migration guide](migration-search-v2.md) before adopting it.

### Search v2 ordering and pagination

- Filter-only search defaults to `external_id ASC`.
- Ranked search defaults to score descending, then `external_id` as a stable
  tie-break.
- Field sort also appends `external_id` as a stable tie-break.
- Page size has a maximum of 1,000.
- Offset has a maximum of 10,000.
- Live keyset cursors are supported.
- A caller can request an optional point-in-time cursor, or PIT.

A cursor is bound to its query shape and result controls. A shape or query
mismatch returns `400 invalid_cursor`. A changed collection UID or active
generation returns `409 stale_cursor`. An expired PIT returns
`410 cursor_expired`.

### Search v2 totals

`track_total` accepts `none`, `up_to`, or `exact`. The default is
`up_to(10000)`. A returned total uses a decimal string and an `eq` or `gte`
relation. When totals are disabled, the total is `null`.

### Search v2 collapse

Collapse is result de-duplication. It does not change source identity.

- A hit keeps its source `external_id`.
- A hit also returns the canonical `collapse_key`.
- `total` is the matching document count before collapse.
- `collapsed_total` is the group count.
- Facets are computed before collapse.
- The representative is the first document under the final hit order.
- Documents with a missing collapse value remain as separate hits.
- The collapse field must be single-valued and `facetable`.
- Its type must be `keyword`, `int64`, or `date`.

The Search v2 source-ID-preserving collapse replaces the current collapse response
contract. See [Search v2 migration](migration-search-v2.md#response-migration).

## Facets and metrics

Exact search facets and metrics are a Search v2 target. The current runtime does not
support this contract.

### Supported target definitions

The target supports:

- terms facets;
- caller-defined range facets; and
- top-level `count`, `valueCount`, `min`, `max`, `sum`, and `avg` metrics.

It does not support:

- general aggregation;
- nested or per-bucket metrics;
- histogram, percentile, cardinality, or pipeline aggregation;
- arbitrary `GROUP BY`;
- facets over kNN or RRF in the first version;
- bucket cursors; or
- facets on `search:all`.

Facet scope is the full match set for the same `query + filter`. Facets run
before pagination, sort, cursor, and collapse. Lumen does not automatically
remove a facet's own filter. A disjunctive navigation UI must send explicit
multi-search requests.

### Terms facets

- Default `size` is 10. Maximum `size` is 100.
- `order` accepts `count_desc`, `value_asc`, or `value_desc`.
- `count_desc` uses `value_asc` as its tie-break.
- A caller can select a canonical value prefix.
- `min_count` defaults to 1.
- The response returns buckets, exact `distinct_value_count`, and `truncated`.
- A caller can request `missing_count`.
- The response does not return `sum_other_doc_count`.

### Range facets

- Each bucket uses a half-open `[from,to)` interval.
- Buckets must be sorted and must not overlap.
- Gaps are valid.
- The first or last bucket can be unbounded.
- Every bucket has a caller-supplied unique and stable key.
- The response returns exact `unbucketed_document_count`.
- A caller can request `missing_count`.

### Multi-value counting

A document contributes at most once to one terms bucket, even if its indexed
multi-value set contained a duplicate before normalization. It can contribute
to several different buckets. The same per-document, per-bucket rule applies
to range facets.

`count` counts documents that have at least one value. `valueCount` counts the
de-duplicated indexed values. `avg` is `sum / valueCount`.

For an empty set, `count` and `valueCount` are `0`. `min`, `max`, `sum`, and
`avg` are `null`.

### Numeric precision

- `int64` and decimal sums are exact and cannot overflow.
- Decimal average has at most 18 fractional digits.
- Decimal average uses round-half-even.
- An average result also returns its exact sum and `valueCount`.
- `float64` uses deterministic compensated summation.
- `float64` sum and average are explicitly marked approximate.
- Every new count value uses a decimal string on the wire.

### Wire shape

The top-level named maps are `facets` and `metrics`. The wire does not use
`aggregations`.

An alias must match `[A-Za-z][A-Za-z0-9_.-]{0,63}`. HTTP field names use
`snake_case`. Every definition and result uses an explicit `kind`
discriminator.

## Limits and failures

These limits are part of the Search v2 exact-or-fail target. The current runtime does
not yet enforce the complete set as one contract.

### Definition and memory limits

- One search, or one complete multi-search request, can contain at most 16
  facet plus metric definitions.
- One terms definition or ranges definition can contain at most 100 buckets.
- One query can use at most 65,536 working buckets.
- Query facet state is limited to 16 MiB on each shard and 16 MiB on the
  coordinator.
- Process-wide facet state is limited to 10% of the Lumen memory budget.

A static limit violation returns `400 query_too_complex`. A temporary capacity
failure returns `429` with `Retry-After`.

### Timeout and failure behavior

`timeout_ms` defaults to 5,000. Valid values are from 1 through 30,000. A
timeout returns `504 query_timeout` and no partial result. Client disconnect
cancels outstanding work. Any shard failure fails the complete search.

Facet and metric results are exact-or-fail. Lumen does not silently return an
approximation when an exact target operation exceeds its allowed resources.

### Read admission

Admission credits equal the number of search items plus facet definitions plus
metric definitions. This makes a multi-search with summaries cost more than a
single simple search.

Managed admission uses runtime UID, KSA namespace, and KSA name as the caller
key. Standalone auth-off uses one anonymous bucket.

### Result cache

The target result cache is one process-wide, byte-weighted LRU.

- Its budget is 5% of internal memory, capped at 256 MiB.
- An entry larger than 1 MiB is not cached.
- Authorization completes before cache lookup.
- Authorized KSAs can share an identical authorized result entry.
- The key includes the semantic request, collection UID, generation, revision,
  PIT, and read consistency.
- The key does not include `timeout_ms`.
- A mutation advances the revision and makes older entries ineligible.

### Facet-value access

Facet values are search metadata. `facetable=true` means every KSA with
runtime query permission can retrieve that field's facet values. Lumen has no
field-level access control list.

This disclosure rule belongs to the planned whole-runtime access contract. It
does not mean Fleet-managed access or facets are implemented.

### Performance completion evidence

The facet roadmap gates will include a single-core CI fixture with 10,000
documents, five facets, and two metrics under 200 ms. A release fixture will
measure 100,000 documents, p95 latency, and peak memory. A change will fail the
gate if latency regresses by more than 20% or memory by more than 10% against
the accepted baseline. Indexing evidence will also report write-throughput and
segment-size change, with review required beyond 30% and 50% respectively.

These numbers are future completion evidence. They are not current service
level objectives.

## Compatibility and migration

Search v2 changes schema types, request fields, totals, sort missing behavior,
collapse, and duplicate discovery. It also adds a Managed capability activation
boundary.

Read [Search v2 migration](migration-search-v2.md) before sending a Search v2
request or schema to a mixed-version runtime. The migration guide owns the
compatibility window and caller actions. This guide owns the final Search v2 query
semantics.

Generated clients do not yet expose typed facets or metrics. They also do not
yet model the complete Search v2 discriminated query and result unions. See
[Generated-client search v2 parity](../ROADMAP.md#generated-client-search-v2-parity).

## Current boundaries

- The current request has one combined query tree. It does not expose the Search v2
  `ScoringQuery` and `FilterExpr` separation.
- The strict Search v2 result ordering, cursor errors, total shape, and collapse
  contract are not supported.
- Exact terms facets, range facets, and top-level metrics are not supported.
- Facet precision, resource governance, distributed convergence, and result
  caching are target behavior.
- Phrase and fuzzy queries in the Search v2 scoring model are not current query
  nodes.
- Generated clients have no typed facet or metric API.
- Managed capability activation for `search_facets_v1` is not supported.

The [support matrix](../STATUS.md#support-matrix) is authoritative. Do not infer
implementation from the detailed target contract in this guide.

## Supporting documents

- [Lumen README](../README.md)
- [Indexing](indexing.md)
- [Search v2 migration](migration-search-v2.md)
- [Protocol](protocol.md)
- [Generated clients](../clients/README.md)
- [Authentication](authentication.md)
- [Client integration](client-integration.md)
- [Current support](../STATUS.md)
- [Future outcomes and non-goals](../ROADMAP.md)
