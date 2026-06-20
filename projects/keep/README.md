# keep

## Brief

Cloud-native, multi-core key-value / claim-check store — the loom/relay data
plane and a Redis / Dragonfly replacement. Promoted from `cclab-kv`: the sharded
engine and tiered RAM+disk persistence are unchanged; the transport is now
**HTTP/2 + OpenAPI** (no raw TCP).

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| KV API | - | implemented | passing | conformance | ready | HTTP/2/OpenAPI scalar, batch, scan, lock, probe, metrics, and claim-check blob API |
| Durability | - | implemented | passing | conformance | ready | WAL-backed durable-before-ack recovery |
| Collections | - | implemented | passing | conformance | ready | hash, set, sorted-set, and list APIs |
| HA / Raft | 121 | implemented | planned | dogfood | not_ready | single-node raftcore path exists; multi-node HTTP/2 network proof remains staged |
| Relay Worker Data Plane | 108 | planned | planned | dogfood | not_ready | worker-facing claim-check contract with relay still needs a closed OpenAPI integration spec |

### KV API

ID: kv-api
Type: Runtime
Surfaces: HTTP: `/v1/kv/*`, `/healthz`, `/readyz`, `/metrics`, `/openapi.json` - public service API.; Rust API: `keep::client::KvClient` - in-tree HTTP client.
EC Dimensions: behavior: `cargo test -p keep --test http_api` - public HTTP API conformance
Root WI: -
Status: passing
Required Verification: conformance
Promise:
Expose a cloud-native key-value and claim-check store over HTTP/2 + OpenAPI,
including scalar get/set/delete, batches, scans, locks, probes, metrics, and
opaque blob roundtrips.
Gate Inventory:
- projects/keep/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| HTTP key-value surface | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| Claim-check blob roundtrip | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| OpenAPI/probe/metrics surface | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |

### Durability

ID: durability
Type: Runtime
Surfaces: Engine: `KvEngine` + persistence - WAL and snapshot-backed state.; HTTP: mutation APIs - durable-before-ack public writes.
EC Dimensions: stability: `cargo test -p keep --test durability` - cold recovery conformance
Root WI: -
Status: passing
Required Verification: conformance
Promise:
Persist mutations before acknowledgement and recover committed state after a
cold restart.
Gate Inventory:
- projects/keep/tests/durability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| WAL-backed cold recovery | epic | - | implemented | passing | conformance | projects/keep/tests/durability.rs |

### Collections

ID: collections
Type: Runtime
Surfaces: HTTP: `/v1/hashes`, `/v1/sets`, `/v1/zsets`, `/v1/lists` - collection APIs.
EC Dimensions: behavior: `cargo test -p keep --test collections_api` - collection operation conformance
Root WI: -
Status: passing
Required Verification: conformance
Promise:
Provide Redis-like hash, set, sorted-set, and list operations on the same
durable engine and HTTP surface.
Gate Inventory:
- projects/keep/tests/collections_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Hash / set / sorted-set operations | epic | - | implemented | passing | conformance | projects/keep/tests/collections_api.rs |
| List push/pop/blocking-pop operations | epic | - | implemented | passing | conformance | projects/keep/tests/collections_api.rs |

### HA / Raft

ID: ha-raft
Type: Runtime
Surfaces: Rust API: `keep::raft` - raftcore-backed state machine.; K8s: StatefulSet - PVC-backed instances.
EC Dimensions: stability: `cargo test -p keep --test raft_node` - raft state-machine conformance
Root WI: 121
Status: auditing
Required Verification: conformance, dogfood
Promise:
Move keep from independent StatefulSet shards toward raft-backed HA without
changing the public KV API.
Gate Inventory:
- projects/keep/tests/raft_node.rs; projects/keep/HA.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Single-node raftcore state machine | epic | 121 | implemented | passing | conformance | projects/keep/tests/raft_node.rs |
| Multi-node HTTP/2 raft network | epic | 121 | planned | planned | dogfood | projects/keep/HA.md |

### Relay Worker Data Plane

ID: relay-worker-data-plane
Type: Runtime
Surfaces: HTTP: keep OpenAPI + relay OpenAPI - worker payload and lease integration contract.
EC Dimensions: behavior: future relay+keep integration gate - worker-facing contract closure
Root WI: 108
Status: auditing
Required Verification: dogfood
Promise:
Serve as the claim-check/value data plane paired with relay's ordered queue and
worker contract.
Gate Inventory:
- projects/keep/README.md; projects/relay/tests/worker_loop.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Relay+keep worker-facing contract | epic | 108 | planned | planned | dogfood | pending integration spec |

## Architecture

| Layer | Module | Notes |
|-------|--------|-------|
| Transport | `src/http/` | axum over hyper — HTTP/1.1 + HTTP/2 cleartext on one port. OpenAPI at `/openapi.json`. |
| Engine | `src/engine.rs` | Sharded, multi-core. Strings/scalars, CAS, leased locks, TTL, lists, hashes, sets, sorted sets. |
| Durability | `src/persistence/` | WAL + snapshot + crash recovery (the disk tier). |

The engine and persistence layers are transport-agnostic. Polyglot workers
integrate against the generated OpenAPI document — there is no language-specific
client to ship (relay+keep worker contract, #108).

## HTTP surface (v1)

| Method | Path | Op |
|--------|------|----|
| GET/PUT/DELETE/HEAD | `/v1/kv/{key}` | get / set / delete / exists |
| POST | `/v1/kv/{key}/incr` | atomic incr/decr (signed `delta`) |
| POST | `/v1/kv/{key}/cas` | compare-and-swap |
| POST | `/v1/kv/{key}/setnx` | set-if-absent |
| POST | `/v1/kv:mget` `:mset` `:mdel` | batch |
| GET | `/v1/kv?prefix=&limit=` | scan |
| POST/DELETE/PATCH | `/v1/locks/{key}` | acquire / release / extend lease |
| POST/GET | `/v1/lists/{key}` + `/{lpush,rpush,lpop,rpop,blpop,brpop}` `/length` | list push/pop/blocking-pop/range/len |
| POST/GET/DELETE | `/v1/hashes/{key}` + `/length` `/mget` `/incr` `/fields/{field}` | hash ops |
| POST/GET/DELETE | `/v1/sets/{key}` + `/length` `/members/{m}` | set ops |
| POST/GET/DELETE | `/v1/zsets/{key}` + `/length` `/incr` `/members/{m}/{score,rank}` | sorted-set ops |
| POST/GET | `/v1/kv/{key}/{expire,ttl,persist,getex}` | TTL / expiry on any key |
| GET | `/healthz` `/readyz` `/metrics` `/info` `/openapi.json` `/docs` | admin / probes |

**Values.** Structured values travel as native JSON (`application/json`, body
`{"value": <json>, "ttl_ms": <opt>}`). Opaque blobs (claim-check payloads) travel
as raw bytes (`application/octet-stream`, TTL via `?ttl_ms=`) and never round-trip
through JSON — `GET` returns them verbatim as octet-stream.

**Durability.** All mutations are WAL-backed and durable-before-ack — scalars
(`/v1/kv`, incr/cas/setnx, `:mset`/`:mdel`), collections (hash / set /
sorted-set / list push+pop), and TTL ops (expire/persist). A write returns 200
only after its op is fsynced (group-committed); committed state survives a cold
recovery (see `tests/durability.rs`).

**Blocking pops.** `BLPOP`/`BRPOP` long-poll up to `timeout_ms` (capped at 60 s)
for an element, waking immediately on a concurrent push via a per-key notifier.

## Run

```bash
cargo run -p keep --bin keep            # listens on 127.0.0.1:7117, ./data
KEEP_HOST=0.0.0.0 KEEP_PORT=7117 keep   # all config via KEEP_* env or --flags
```

Config: `--host/--port/--shards/--data-dir/--disable-persistence/--fsync-ms/
--snapshot-secs/--snapshot-ops/--body-limit/--grace-secs`, each with a `KEEP_*`
env fallback. `RUST_LOG` overrides `--log-level`.

## Kubernetes

`StatefulSet` (each pod owns a PVC-backed disk tier) + headless & ClusterIP
Services + ConfigMap + PDB, distroless nonroot image. SIGTERM flips `/readyz` to
503, drains the grace window, then flushes the WAL.

```bash
kubectl apply -k k8s/overlays/dev      # or staging / prod
```

Images: `Dockerfile` (from-source, build context = repo root) and
`Dockerfile.release` (published binary into distroless).

## Status / roadmap

- ✅ HTTP/2 + OpenAPI surface, k8s-native (probes, SIGTERM drain, env config) — #114
- ✅ durable-before-ack WAL (no-drop, group commit) — #114
- ✅ perf-gate via meter: engine throughput ratchet + server resource gate — see [PERF-GATE.md](PERF-GATE.md) (#126). Competitor comparison (vs Redis/Dragonfly) is the separate one-off `examples/bench_compare.rs`.
- ☐ worker-facing OpenAPI contract finalized with relay — #108
- ◑ HA — phase A (sharded scale-out + `/cluster`) done; phase C raft via openraft integrated single-node (`--features raft`, engine-backed state machine, proven by `tests/raft_node.rs`) — multi-node HTTP/2 network + durable raft log staged; see [HA.md](HA.md) (#121)
- ✅ queuekit and queue `ion` feature consumers now use `keep::client` instead
  of the retired raw-TCP `cclab-kv` client; remaining cleanup is deduping the
  legacy `crates/cclab-kv` + `projects/queue/kv` copies.
