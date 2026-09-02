# Indexing and rebuild

## Indexing and rebuild

- Problem: Callers need safe current index writes and rebuild behavior.
- Who: Application teams that own source records.
- Promise: Lumen accepts the declared index-write API and keeps source records outside Lumen.
- Status rows: `index-query-api`, `current-index-write-api`, `current-external-version-writes`, `current-request-id-deduplication`, `item-atomic-batch-writes`, `versioned-delete-tombstones`, `shadow-rebuild-generations`, `strict-search-schema-types`.
- Limits today: Item atomicity, tombstones, shadow generations, and strict target types are future work.
- Non-goals: Source-record ownership or hydration.
- Neighbours: Querying selects the indexed IDs; recovery defines durable acknowledgement.

## Durable write contract (Milestone #9)

- Problem: Persistent backends need one public acknowledgement meaning.
- Who: Writers that retry after faults.
- Promise: Persistent writes acknowledge only after durable commit and index apply.
- Outcome: `durable-write-contract`. Tracking: [Milestone #9](https://github.com/chrischeng-c4/axiom/milestone/9).
- Non-goals: Making Lumen the source of truth.
- Open: Define one contract across every selected persistent backend.
- Neighbours: Idempotency and recovery consume this acknowledgement.

## Idempotent write replay (Milestone #10)

- Problem: A retry can otherwise apply a write twice.
- Who: HTTP and generated-client writers.
- Promise: A durable payload-bound key replays the first result safely.
- Outcome: `idempotent-write-replay`. Tracking: [Milestone #10](https://github.com/chrischeng-c4/axiom/milestone/10).
- Non-goals: Unbounded key retention.
- Open: Define retention and conflict reporting.
- Neighbours: Durable writes and generated-client resilience.

## Item-atomic batch writes

- Problem: One failed field must not leave part of an item visible.
- Who: Batch-write callers.
- Promise: Each accepted item becomes visible in full or not at all.
- Outcome: `item-atomic-batch-writes`. Tracking: Not assigned.
- Non-goals: A transaction across unrelated batch items.
- Open: Define stable per-item results and retries.
- Neighbours: Idempotent replay and versioned tombstones.

## Shadow rebuild generations

- Problem: A schema conversion must not replace the active index early.
- Who: Teams evolving indexed schemas.
- Promise: A validated shadow generation can become active atomically.
- Outcome: `shadow-rebuild-generations`. Tracking: Not assigned.
- Non-goals: In-place destructive schema changes.
- Open: Define retained-generation limits and rollback detail.
- Neighbours: Strict schema types and Search v2 migration.

## Strict search schema types

- Problem: Legacy number and set types cannot express the target schema safely.
- Who: Schema authors.
- Promise: Collections use explicit scalar types with independent multi-value and facet options.
- Outcome: `strict-search-schema-types`. Tracking: Not assigned.
- Non-goals: Silent rounding or schema coercion.
- Open: Define each target type and migration refusal.
- Neighbours: Shadow rebuild generations and Search v2 migration.

## Non-goals in this area

Lumen does not own caller source documents.
