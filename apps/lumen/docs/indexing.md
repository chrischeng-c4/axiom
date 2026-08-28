# Lumen indexing

## Purpose

This guide defines the Lumen indexing contract. It owns source-data
responsibility, schema meaning, writes, durability, rebuild, and activation.

The first part of each section states current behavior. Text marked as the
0.5 target is future work. It is not available in the current runtime. Use the
[support matrix](../STATUS.md) for the current support state and the
[roadmap](../ROADMAP.md) for completion evidence.

## Contract map

| Fact | Canonical source | Discovery |
|---|---|---|
| Current HTTP methods and wire schemas | [`lumen spec` OpenAPI](../clients/openapi.json) | Run `lumen spec` or request `GET /openapi.json`. |
| Source records and derived-index ownership | [Data ownership](#data-ownership) | Read this guide before designing ingest or hydration. |
| Current and target field schema meaning | [Schema contract](#schema-contract) | Run `lumen spec --fields` for current fields. |
| Current and target write semantics | [Write contract](#write-contract) | Run `lumen spec --shapes` for current request bodies. |
| Acknowledgement and persistence meaning | [Durability](#durability) | Check the selected backend and fsync policy. |
| Rebuild, generation, activation, and rollback | [Rebuild and activation](#rebuild-and-activation) | Separate the current stream endpoint from the 0.5 target. |
| 0.4.x to 0.5.0 caller changes | [0.5 search migration](migration-0.5-search.md) | Use the versioned migration table and future offline tools. |
| Current implementation support | [STATUS.md](../STATUS.md) | Read each indexing support row and its evidence. |
| Future outcomes | [ROADMAP.md](../ROADMAP.md) | Follow the outcome linked from a current limit. |

## Data ownership

The caller's database or object store is the source of truth. Lumen stores a
derived search index. It does not become the source record store.

The caller owns:

- every `external_id`;
- source writes, CDC or outbox checkpoints, source-delivery retry, and
  freshness;
- the mapping from a source record to indexed fields;
- execution of any embedding model and creation of raw vectors or perceptual
  hashes;
- rebuild input and source-record retention; and
- source-record hydration after a search.

Lumen owns:

- declared schema validation;
- deterministic mutation of the derived index;
- query-visible index state;
- shard and replica placement inside a Lumen runtime; and
- recovery of the derived index within its stated durability contract.

Docker with its persistent named volume provides a durable backend boundary.
There is no uniform durable-acknowledgement contract across all backends.

The planned generated client owns the mechanics of one Lumen HTTP request. It
supplies typed errors, deadline and cancellation, safe retry, and idempotency
inputs. That support does not take ownership of the caller's source transaction,
CDC checkpoint, or delivery policy.

The caller must be able to rebuild Lumen from the source of truth. Lumen can
retain a durable derived index, but that retention does not transfer source
ownership.

## Schema contract

### Current schema

The current public fields are `text`, `keyword`, `number`, `set`, `vector`, and
`hash`. The [README example](../README.md#primary-workflow) shows a current
schema. Adding a new field is an online extension. Changing an existing field
type is rejected.

Lumen indexes caller-supplied raw vectors and perceptual hashes. It does not run
an embedding model.

The current `number` and `set` fields do not provide the strict type and
orthogonal multi-value contract below.

### 0.5 target schema

The 0.5 schema separates storage type from multi-value and facet behavior.

| Type | Contract |
|---|---|
| `text` | Analyzed text for lexical search. |
| `keyword` | One exact UTF-8 value before orthogonal options are applied. |
| `int64` | A signed 64-bit integer. |
| `float64` | An IEEE 754 binary floating-point value. |
| `decimal(p,s)` | An exact decimal with declared precision and scale. |
| `timestamp` | One timestamp value with the 0.5 canonical wire form. |
| `date` | One calendar date with the 0.5 canonical wire form. |
| `boolean` | One true or false value. |
| `vector` | A fixed-length vector with a declared metric and backend. |
| `hash` | A caller-computed hash used by Hamming search. |

`multi` and `facetable` are orthogonal field options. They are not field
types. A multi-valued field is a de-duplicated, unordered set. `null`, an
omitted field, and an empty multi-value set all mean missing.

For `decimal(p,s)`, precision is from 1 through 38. Scale is from 0 through the
declared precision. The wire value is a canonical decimal string without an
exponent. Lumen rejects a value that exceeds the declared precision or scale.
It does not round the value automatically.

Changing an existing field from `facetable=false` to `facetable=true` needs a
shadow rebuild. A new field with no historical values can be added as
facetable without rebuilding older records.

The strict schema is a target contract. See
[Strict search schema types](../ROADMAP.md#strict-search-schema-types). The
current runtime still accepts the current field model.

## Write contract

### Current writes

`POST /collections/{id}/index` is a field merge. Each item replaces one
`(external_id, field)` value. Fields not present in the request stay unchanged.

`PUT /collections/{id}/docs:replace` is full document replacement. It replaces
all indexed fields for the supplied `external_id`. Fields omitted from the
replacement are removed from that indexed row.

The current request body can carry `request_id`. Its de-duplication window is
five minutes and its state is process-local. The key is not bound to a payload.
Lumen does not replay the original response for a duplicate key. A restart or
another process can therefore accept the same key again.

Current merge items can carry a field external version. Current document
replacement can carry a document external version. These mechanisms provide
partial last-write-wins protection against older arrivals. They do not yet
provide one collection-wide ownership and versioning contract.

The current batch behavior is not the 0.5 item-atomic partial-success
contract. A caller must not infer a stable per-item commit and error model from
the current batch response.

### 0.5 target writes

HTTP writes use `Idempotency-Key`. Generated clients create a key by default,
and the caller can override it. Lumen retains a key for at least 24 hours.

- The same key and the same payload replay the original response.
- The same key and a different payload return `409`.
- The retained record survives the process boundary required by the selected
  persistent runtime.

A batch write has item-atomic partial success. Each item either applies in full
or reports its own failure. A failed item does not make an accepted sibling
item partial.

Each collection selects one write-ownership model:

- Document ownership replaces the complete indexed document.
- Field ownership merges independently owned indexed fields.

Each collection also selects one ordering model:

- Arrival order accepts the newest accepted arrival.
- External version accepts only a mutation newer than the recorded external
  version.

A versioned delete records a tombstone. The tombstone prevents an older write
from making deleted data visible again.

These behaviors are future outcomes. See
[Idempotent write replay](../ROADMAP.md#idempotent-write-replay),
[Item-atomic batch writes](../ROADMAP.md#item-atomic-batch-writes), and
[Versioned deletes and tombstones](../ROADMAP.md#versioned-deletes-and-tombstones).

## Durability

### Current acknowledgement

The meaning of a current successful write depends on the storage backend and
fsync policy. In-memory mode has no restart durability. Persistent backends
have different commit, apply, and fsync boundaries. The current API does not
give one uniform durable-acknowledgement promise for every mode.

A caller must select and verify the runtime storage configuration. A current
2xx response must not be treated as the same disk-loss guarantee in every
backend mode.

### 0.5 target acknowledgement

Persistent mode returns 2xx only after the durable commit and index apply both
complete. The contract names the failure boundary that remains after that
response. In-memory mode is explicitly marked `ephemeral`; it does not make a
durability promise.

See [Durable write contract](../ROADMAP.md#durable-write-contract) for the
required crash, restart, and acknowledgement evidence.

## Rebuild and activation

### Current rebuild

`POST /collections/{id}/reindex/stream` accepts a streamed rebuild. It writes
directly into the active collection. The current runtime has no shadow
generation, seal step, atomic activation, or generation rollback.

A current rebuild can therefore affect the active index while the stream is
still running. The endpoint does not provide the 0.5 dual-write and activation
contract.

### 0.5 target rebuild

A rebuild creates a shadow generation while the current generation remains
active. Active and shadow generations receive the same live writes.

Before a live write changes either generation, Lumen validates it against both
schemas and both generation constraints. If either generation rejects it,
neither generation is changed.

Rebuild input uses ordered chunks and one durable Operation resource. The
operation records progress and can resume without guessing the last accepted
chunk. Cancelling the rebuild returns the collection to active-only writes.

The shadow generation must be explicitly sealed before activation. Activation
uses an ETag compare-and-swap operation. Rollback uses the same ETag guard, so
a stale operator cannot replace a newer generation decision.

The previous generation is retained for 24 hours by default. A caller can
choose a retention period from one hour through seven days. After that period,
rollback is no longer promised.

See [Shadow rebuild generations](../ROADMAP.md#shadow-rebuild-generations). None
of these shadow-generation behaviors are implemented today.

## Current boundaries

- The current `request_id` mechanism is short-lived, process-local, not
  payload-bound, and does not replay the original response.
- Current external versions provide partial last-write-wins behavior. They do
  not yet provide versioned delete tombstones.
- Current batch writes do not have the complete item-atomic partial-success
  contract.
- Write acknowledgement is not uniform across in-memory and persistent modes.
- Stream reindex writes to the active collection. There is no shadow
  generation, seal, activation, or rollback.
- The strict 0.5 schema types and orthogonal `multi` and `facetable` options are
  not supported.

The [support matrix](../STATUS.md#support-matrix) is authoritative for these
states. The target text in this guide is a design contract, not proof of
implementation.

## Supporting documents

- [Lumen README](../README.md)
- [Querying](querying.md)
- [0.5 search migration](migration-0.5-search.md)
- [Protocol](protocol.md)
- [Current support](../STATUS.md)
- [Future outcomes and non-goals](../ROADMAP.md)
- [Generated clients](../clients/README.md)
- [Client integration](client-integration.md)
