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
| CLI Interface | - | implemented | passing | conformance | not_ready | keep binary exposes HTTP/2/OpenAPI, probes, metrics, env/flag config, and graceful drain |
| Competitive Broker Feature Parity | 108 | implemented | planned | dogfood | not_ready | Redis/Dragonfly-like KV, collections, locks, TTL, and claim-check API exist; relay worker contract remains open |
| Competitive Broker Performance | 126 | implemented | planned | dogfood | not_ready | engine perf gate and competitor comparison exist, but external Redis/Dragonfly comparison is not release-closed |
| Long-Running Stability | 121 | implemented | planned | dogfood | not_ready | WAL/snapshot recovery and graceful drain pass locally; multi-node raft network remains staged |
| Security Hardening | - | planned | planned | negative | not_ready | body limits exist, but auth/TLS/negative security gates are not yet defined |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Surfaces: CLI: `keep` - long-running HTTP/2 key-value service process.; HTTP: `/openapi.json`, `/healthz`, `/readyz`, `/metrics` - binary-served operational surface.
EC Dimensions: behavior: `cargo test -p keep --test http_api` - binary-facing OpenAPI/probe/metrics and API conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Expose Keep as a runnable long-lived binary with stable config, HTTP/2/OpenAPI,
health/readiness probes, metrics, and graceful drain behavior.
Gate Inventory:
- projects/keep/tests/http_api.rs; projects/keep/src/bin/keep.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Keep process interface | epic | - | implemented | passing | conformance | projects/keep/src/bin/keep.rs; projects/keep/tests/http_api.rs |
| OpenAPI/probe/metrics surface | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| Graceful drain readiness flip | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |

### Competitive Broker Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Surfaces: HTTP: `/v1/kv/*`, `/v1/hashes`, `/v1/sets`, `/v1/zsets`, `/v1/lists`, `/v1/locks` - Redis/Dragonfly-style data plane over HTTP/2.; Rust API: `keep::client::KvClient` - in-tree HTTP client.
EC Dimensions: behavior: `cargo test -p keep --test http_api --test collections_api` - public KV, collection, lock, TTL, and claim-check conformance
Root WI: 108
Status: auditing
Required Verification: conformance, dogfood
Promise:
Cover the core functions Keep needs to replace Redis/Dragonfly in Axiom
workloads: scalar KV, claim-check blobs, TTL/locks, scans, and collection
operations over a cloud-native HTTP/2/OpenAPI surface.
Gate Inventory:
- projects/keep/tests/http_api.rs; projects/keep/tests/collections_api.rs; projects/keep/README.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| HTTP key-value surface | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| Claim-check blob roundtrip | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| Hash / set / sorted-set operations | epic | - | implemented | passing | conformance | projects/keep/tests/collections_api.rs |
| List push/pop/blocking-pop operations | epic | - | implemented | passing | conformance | projects/keep/tests/collections_api.rs |
| Relay+keep worker-facing contract | epic | 108 | planned | planned | dogfood | pending integration spec |

### Competitive Broker Performance

ID: competitor-performance
Type: RuntimeTool
Surfaces: Example: `bench_compare` - Redis/Dragonfly comparison harness.; Meter: `PERF-GATE.md` - engine throughput and resource gate.
EC Dimensions: efficiency: `meter` - engine throughput and resource ratchet; behavior: `cargo test -p keep` - API behavior under the performance-relevant surfaces
Root WI: 126
Status: auditing
Required Verification: conformance, dogfood
Promise:
Keep performance claims tied to repeatable engine/resource gates and an
external Redis/Dragonfly comparison, not anecdotal local timings.
Gate Inventory:
- projects/keep/PERF-GATE.md; projects/keep/examples/bench_compare.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Engine throughput ratchet | epic | 126 | implemented | planned | conformance | projects/keep/PERF-GATE.md |
| Redis/Dragonfly comparison | epic | 126 | implemented | planned | dogfood | projects/keep/examples/bench_compare.rs |

### Long-Running Stability

ID: long-running-stability
Type: RuntimeTool
Surfaces: CLI: `keep` - long-running WAL/snapshot-backed service process.; K8s: `projects/keep/k8s` - StatefulSet/PDB deployment shape.; Rust API: `keep::raft` - raftcore-backed HA path.
EC Dimensions: stability: `cargo test -p keep --test durability --test http_api --test raft_node` - recovery, drain, probe, and raft state-machine conformance
Root WI: 121
Status: auditing
Required Verification: conformance, dogfood
Promise:
Run as a long-lived data-plane service without losing durable writes across
restart, without receiving traffic during drain, and with a path to raft-backed
HA that preserves the public KV API.
Gate Inventory:
- projects/keep/tests/durability.rs; projects/keep/tests/http_api.rs; projects/keep/tests/raft_node.rs; projects/keep/HA.md; projects/keep/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| WAL-backed cold recovery | epic | - | implemented | passing | conformance | projects/keep/tests/durability.rs |
| Snapshot and final flush lifecycle | epic | - | implemented | passing | conformance | projects/keep/src/persistence; projects/keep/tests/durability.rs |
| Graceful shutdown and readiness drain | epic | - | implemented | passing | conformance | projects/keep/src/bin/keep.rs; projects/keep/tests/http_api.rs |
| Single-node raftcore state machine | epic | 121 | implemented | passing | conformance | projects/keep/tests/raft_node.rs |
| Multi-node HTTP/2 raft network | epic | 121 | planned | planned | dogfood | projects/keep/HA.md |

### Security Hardening

ID: security-hardening
Type: RuntimeTool
Surfaces: HTTP: keep public API - request body and data-plane boundary.; K8s: `projects/keep/k8s` - deployment boundary for future network policy and identity.
EC Dimensions: security: `guard` - negative API and deployment security gate to be authored; behavior: `cargo test -p keep --test http_api` - body-limit and public-route smoke
Root WI: -
Status: auditing
Required Verification: negative
Promise:
Keep the long-running KV service safe by enforcing request boundaries and
adding explicit negative gates for authn/z, TLS, network policy, and malformed
or oversized request handling before production readiness.
Gate Inventory:
- projects/keep/src/http/routes.rs; projects/keep/tests/http_api.rs; pending guard/negative security inventory

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Body limit and public route boundary | epic | - | implemented | passing | smoke | projects/keep/src/http/routes.rs; projects/keep/tests/http_api.rs |
| Auth/TLS/network-policy boundary | epic | - | planned | planned | negative | pending guard/negative security inventory |
| Malformed and oversized request negative tests | epic | - | planned | planned | negative | pending guard/negative security inventory |

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
