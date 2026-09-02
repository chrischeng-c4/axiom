# Indexing and rebuild

## Indexing and rebuild

- Problem: Callers need safe current index writes and rebuild behavior.
- Who: Application teams that own source records.
- Promise: Lumen accepts the declared index-write API and keeps source records outside Lumen.
- Status rows: `index-query-api`, `current-index-write-api`, `current-external-version-writes`, `current-request-id-deduplication`, `item-atomic-batch-writes`, `versioned-delete-tombstones`, `shadow-rebuild-generations`, `strict-search-schema-types`.
- Limits today: Item atomicity, tombstones, shadow generations, and strict target types are future work.
- Non-goals: Source-record ownership or hydration.
- Neighbours: Querying selects the indexed IDs; recovery defines durable acknowledgement.

## Non-goals in this area

Lumen does not own caller source documents.
