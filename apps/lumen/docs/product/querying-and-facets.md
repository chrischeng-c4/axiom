# Querying and facets

## Querying and facets

- Problem: Callers need current search behavior with explicit future limits.
- Who: Applications that search and hydrate caller-owned records.
- Promise: Lumen returns ranked external IDs for current exact, lexical, vector, hash, and duplicate search.
- Status rows: `raw-vector-and-hash-search`, `embedding-model-execution`, `unified-scoring-filter-model`, `search-result-contract-v2`, `exact-search-facets-metrics`, `facet-resource-governance`, `routed-facet-convergence`.
- Limits today: Lumen does not run embedding models and does not provide the Search v2 query or facet contract.
- Non-goals: Source-record hydration and embedding execution.
- Neighbours: Protocol documents the wire contract; indexing owns schema changes.

## Unified search contract

- Problem: Scoring, filtering, cursors, and totals need one deterministic contract.
- Who: Search callers and generated clients.
- Promise: Search separates scoring and filtering with stable result controls.
- Outcome: `unified-search-contract`. Tracking: Not assigned.
- Non-goals: Cross-collection joins.
- Open: Finalize compatibility and failure semantics.
- Neighbours: Search v2 migration and generated-client parity.

## Exact search facets metrics

- Problem: Callers need exact reported facet and metric values.
- Who: Search dashboards and APIs.
- Promise: Search can return declared exact facets and metrics.
- Outcome: `exact-search-facets-metrics`. Tracking: Not assigned.
- Non-goals: General OLAP.
- Open: Define exact limits and failures.
- Neighbours: Facet governance and distributed convergence.

## Non-goals in this area

Lumen does not execute embedding models or become an OLAP engine.
