# Querying and facets

## Querying and facets

- Problem: Callers need current search behavior with explicit future limits.
- Who: Applications that search and hydrate caller-owned records.
- Promise: Lumen returns ranked external IDs for current exact, lexical, vector, hash, and duplicate search.
- Status rows: `raw-vector-and-hash-search`, `embedding-model-execution`, `unified-scoring-filter-model`, `search-result-contract-v2`, `exact-search-facets-metrics`, `facet-resource-governance`, `routed-facet-convergence`.
- Limits today: Lumen does not run embedding models and does not provide the Search v2 query or facet contract.
- Non-goals: Source-record hydration and embedding execution.
- Neighbours: Protocol documents the wire contract; indexing owns schema changes.

## Non-goals in this area

Lumen does not execute embedding models or become an OLAP engine.
