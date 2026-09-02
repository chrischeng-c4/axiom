# Indexing and rebuild

## Indexing and rebuild

- Problem: Callers need safe current index writes and rebuild behavior.
- Who: Application teams that own source records.
- Promise: Lumen accepts the declared index-write API and keeps source records outside Lumen.
- Status rows: `index-query-api`, `current-index-write-api`, `current-external-version-writes`, `current-request-id-deduplication`, `item-atomic-batch-writes`, `versioned-delete-tombstones`, `shadow-rebuild-generations`, `strict-search-schema-types`.
- Limits today: Item atomicity, tombstones, shadow generations, and strict target types are future work.
- Non-goals: Source-record ownership or hydration.
- Neighbours: Querying selects the indexed IDs; recovery defines durable acknowledgement.

## Durable write contract

- Problem: Persistent backends need one public acknowledgement meaning.
- Who: Writers that retry after faults.
- Promise: Persistent writes acknowledge only after durable commit and index apply.
- Outcome: `durable-write-contract`. Tracking: Not assigned.
- Non-goals: Making Lumen the source of truth.
- Open: Define one contract across every selected persistent backend.
- Neighbours: Idempotency and recovery consume this acknowledgement.

## Idempotent write replay

- Problem: A retry can otherwise apply a write twice.
- Who: HTTP and generated-client writers.
- Promise: A durable payload-bound key replays the first result safely.
- Outcome: `idempotent-write-replay`. Tracking: Not assigned.
- Non-goals: Unbounded key retention.
- Open: Define retention and conflict reporting.
- Neighbours: Durable writes and generated-client resilience.

## Non-goals in this area

Lumen does not own caller source documents.
