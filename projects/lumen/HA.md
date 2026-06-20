# lumen — high availability & replication

## Decision: lumen does NOT run consensus (no Raft)

lumen is a **log-replicated, derived, rebuildable search index** — explicitly
*not* a system of record (see the README "Brief": the caller owns the source of
truth; lumen indexes the caller's `external_id`s). Every write is **published to
a log**, not applied where it lands; every serving pod **tails that log** and
folds it into its own index. State is *lossable but rebuildable* from the log +
the caller.

Because lumen is not a source of truth, it does **not** need Raft for read/write:

- **Ordering + durability already belong to the broker (the log).** A write goes
  to the broker; the broker is the ordering/durability authority. Putting a
  second consensus layer *inside* lumen would duplicate what the already-ordered
  log provides.
- **Writes don't "land" on lumen**, so there is no lumen-side write quorum to
  coordinate; reads are served per-pod from the locally-folded index
  (eventually-consistent / bounded-staleness — appropriate for a search index).
- **Rebuildable** ⇒ lumen needs no quorum durability for its own state.

Rule of thumb across the ecosystem: *needs Raft iff the service is a source of
truth.* relay (the log/broker) — yes. keep (KV system of record) — yes. **lumen
(derived index) — no.** openraft has already been retired here.

### The `raft` module is a replica *cluster view*, not consensus

`src/raft.rs` (CODEGEN, from `tech-design/.../projects-lumen-src-raft-rs.md`)
provides the cluster-view DTOs the API serves on `/debug/cluster`: peer DNS map,
per-pod role, `applied_index` / `replication_lag_ms`, and the
`X-Read-Consistency` parsing (`leader` / `bounded(ms)` / `any`). These remain
legitimate for a log-tailing replica (which peers exist, how fresh a read is).

**`撤 skeleton` (remaining, via the aw spec):** the module's *framing* still
reads like a consensus stub ("until the openraft wiring lands", "stub: pod 0
always claims leader"). Since the file is CODEGEN-managed, the correct fix is to
reframe its **spec** (drop the consensus-pending language; rename the surface to
a `cluster` view) and regenerate — not a hand-edit of the CODEGEN region. The
`RaftRole` enum collapses to a replica role (a single writer/leader is just "the
pod the broker write went through").

## #124 — broker log-tailing moves NATS → relay broadcast

lumen **already tails a broker** via the `WalLog` trait
(`publish`/`subscribe(from_seq)`/`latest_seq`); today the backend is NATS
JetStream (`src/wal_nats.rs`, `--wal nats`). #124 swaps the broker to **relay
broadcast** (relay is now itself HA via `libs/raftcore`), so each lumen pod tails
relay's ordered, replicated log and folds it into its index.

**Backend shape (`src/wal_relay.rs`, the remaining slice):**

- `publish(record)` → `POST {relay}/v1/{subject}/publish` with
  `payload = json(WalRecord)`; the returned `AppendOutcome.seq (+1)` is the WAL seq.
- `subscribe(from)` → `GET {relay}/v1/{subject}/subscribe?from_seq={from}`; decode
  relay's length-prefixed CBOR `LogEntry` frames (reuse `relay::wire::decode_frames`)
  and yield `(seq+1, WalRecord)` — exactly the tail loop `relay::spawn_follower`
  already proved.
- `latest_seq` → replay-from-offset (relay exposes no length endpoint; a derived
  index is rebuildable, so cold start replays from the requested offset).

**Constraint — keep the serving binary openssl-free.** lumen deliberately ships
no HTTP client in the serving binary (reqwest is dev-only) for a clean
cross-compile. relay is **h2c (cleartext)**, so the relay backend must use a
**plaintext HTTP/2 client with no TLS** — built on `hyper` (already a lumen
dependency via axum), **not** reqwest — so no openssl/rustls is linked.

Wiring: add `WalBackend::Relay { --relay-url }` alongside `Embedded` / `Nats` in
`src/bin/lumen.rs`; everything downstream (`WriteCoordinator`, the fold loop) is
unchanged because it only depends on the `WalLog` trait.

## Status

- ✅ Decision recorded: lumen tails the broker; no consensus.
- ⬜ `撤 skeleton`: reframe the CODEGEN `raft` spec → `cluster` view, regenerate.
- ⬜ #124 `RelayWal`: hyper-based (h2c, no TLS) `WalLog` backend + `--wal relay` +
  an in-process-relay round-trip test.
