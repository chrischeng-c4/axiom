# keep

## Brief

Cloud-native, multi-core key-value / claim-check store — the loom/relay data
plane and a Redis / Dragonfly replacement. Promoted from `cclab-kv`: the sharded
engine and tiered RAM+disk persistence are unchanged; the transport is now
**HTTP/2 + OpenAPI** (no raw TCP).

## Capabilities

The RuntimeTool baseline capabilities selected by `aw.toml` are mandatory for
this long-running service class. They do not replace Keep's product
capabilities; KV, collections, durability, HA, and the relay data plane remain
first-class domain roots.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| CLI Interface | - | implemented | passing | conformance | not_ready | mandatory baseline: binary, OpenAPI, probes, metrics, and graceful drain |
| Competitive KV Feature Parity | 108 | implemented | planned | dogfood | not_ready | mandatory baseline: Redis/Dragonfly replacement breadth remains open on relay worker contract |
| Competitive KV Performance | 126 | implemented | planned | dogfood | not_ready | mandatory baseline: engine ratchet and external Redis/Dragonfly comparison are not release-closed |
| Long-Running Stability | 121 | implemented | planned | dogfood | not_ready | mandatory baseline: durable recovery and drain pass locally; multi-node raft remains staged |
| Security Hardening | - | planned | planned | negative | not_ready | mandatory baseline: request limits exist; auth/TLS/negative gates are still open |
| HTTP/2 API List | - | implemented | passing | conformance | not_ready | mandatory baseline: concise HTTP/2 route list, OpenAPI pointer, probes, and metrics |
| Kubernetes-Native Deployment | - | implemented | passing | conformance | not_ready | mandatory baseline: kustomize manifests, probes, drain, env config, and nonroot image |
| Primary Replicas | 121 | planned | planned | dogfood | not_ready | mandatory baseline: raft-backed primary/replica topology and failover remains staged |
| KV API | - | implemented | passing | conformance | not_ready | domain: scalar KV, batch, scan, locks, probes, metrics, and claim-check blobs |
| Collections | - | implemented | passing | conformance | not_ready | domain: hash, set, sorted-set, and list APIs |
| Durability | - | implemented | passing | conformance | not_ready | domain: WAL-backed durable-before-ack recovery |
| HA / Raft | 121 | implemented | planned | dogfood | not_ready | domain: raft-backed HA without changing the public KV API |
| Relay Worker Data Plane | 108 | planned | planned | dogfood | not_ready | domain: claim-check/value plane paired with relay's ordered queue contract |

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
| keep-process-interface | epic | - | implemented | passing | conformance | projects/keep/src/bin/keep.rs; projects/keep/tests/http_api.rs |
| openapi-probe-metrics-surface | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| graceful-drain-readiness-flip | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |

### Competitive KV Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Surfaces: HTTP: `/v1/kv/*`, `/v1/hashes`, `/v1/sets`, `/v1/zsets`, `/v1/lists`, `/v1/locks` - Redis/Dragonfly-style data plane over HTTP/2.; Rust API: `keep::client::KvClient` - in-tree HTTP client.
EC Dimensions: behavior: `cargo test -p keep --test http_api --test collections_api` - public KV, collection, lock, TTL, and claim-check conformance
Root WI: 108
Status: auditing
Required Verification: conformance, dogfood
Promise:
Keep covers the Redis/Dragonfly replacement breadth expected from this runtime
class: scalar KV, claim-check blobs, TTL/locks, scans, collections, and the
relay-facing worker data-plane integration.
Gate Inventory:
- projects/keep/tests/http_api.rs; projects/keep/tests/collections_api.rs; projects/keep/README.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| redis-dragonfly-api-breadth | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs; projects/keep/tests/collections_api.rs |
| http-key-value-surface | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| hash-set-sorted-set-operations | epic | - | implemented | passing | conformance | projects/keep/tests/collections_api.rs |
| relay-keep-worker-facing-contract | epic | 108 | planned | planned | dogfood | pending integration spec |

### Competitive KV Performance

ID: competitor-performance
Type: RuntimeTool
Surfaces: Meter/Vat: `projects/keep/vat.toml#meter-efficiency` - isolated meter execution for performance-relevant API and durability gates.; Example: `bench_compare` - Redis/Dragonfly comparison harness.; Meter: `PERF-GATE.md` - engine throughput and resource gate.
EC Dimensions: efficiency: `cd projects/keep && ../../target/debug/vat run meter-efficiency` - meter-owned runtime evidence inside vat; behavior: `cargo test -p keep` - API behavior under the performance-relevant surfaces
Root WI: 126
Status: auditing
Required Verification: conformance, dogfood
Promise:
Keep performance claims stay tied to repeatable engine/resource gates and an
external Redis/Dragonfly comparison, not anecdotal local timings.
Gate Inventory:
- projects/keep/vat.toml; projects/keep/PERF-GATE.md; projects/keep/examples/bench_compare.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| vat-meter-runtime-gate | epic | 126 | implemented | planned | conformance | projects/keep/vat.toml#meter-efficiency |
| engine-throughput-ratchet | epic | 126 | implemented | planned | conformance | projects/keep/PERF-GATE.md |
| redis-dragonfly-comparison | epic | 126 | implemented | planned | dogfood | projects/keep/examples/bench_compare.rs |

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
| wal-backed-cold-recovery | epic | - | implemented | passing | conformance | projects/keep/tests/durability.rs |
| graceful-shutdown-and-readiness-drain | epic | - | implemented | passing | conformance | projects/keep/src/bin/keep.rs; projects/keep/tests/http_api.rs |
| multi-node-http-2-raft-network | epic | 121 | planned | planned | dogfood | projects/keep/HA.md |

### Security Hardening

ID: security-hardening
Type: RuntimeTool
Surfaces: Guard/Vat: `projects/keep/vat.toml#guard-security` - isolated guard scan with meter runtime evidence.; HTTP: keep public API - request body and data-plane boundary.; K8s: `projects/keep/k8s` - deployment boundary for future network policy and identity.
EC Dimensions: security: `cd projects/keep && ../../target/debug/vat run guard-security` - guard-owned static/runtime evidence; behavior: `cargo test -p keep --test http_api` - body-limit and public-route smoke
Root WI: -
Status: auditing
Required Verification: negative
Promise:
Keep the long-running KV service safe by enforcing request boundaries and
adding explicit negative gates for authn/z, TLS, network policy, and malformed
or oversized request handling before production readiness.
Gate Inventory:
- projects/keep/vat.toml; projects/keep/src/http/routes.rs; projects/keep/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| body-limit-and-public-route-boundary | epic | - | implemented | passing | smoke | projects/keep/src/http/routes.rs; projects/keep/tests/http_api.rs |
| auth-tls-network-policy-boundary | epic | - | planned | planned | negative | pending guard/negative security inventory |
| guard-static-runtime-evidence | epic | - | implemented | planned | negative | projects/keep/vat.toml#guard-security |
| malformed-and-oversized-request-negative-tests | epic | - | planned | planned | negative | projects/keep/vat.toml#guard-security |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Surfaces: HTTP: `/v1/kv/*`, `/v1/hashes`, `/v1/sets`, `/v1/zsets`, `/v1/lists`, `/v1/locks`, `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` - concise HTTP/2 API list for operators and client authors.
EC Dimensions: behavior: `cargo test -p keep --test http_api --test collections_api` - public route list and data-plane conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Publish the supported HTTP/2 API surface as a compact endpoint inventory, with
probe, metrics, and OpenAPI pointers, without making OpenAPI completeness the
capability definition.
Gate Inventory:
- projects/keep/README.md#http-surface-v1; projects/keep/tests/http_api.rs; projects/keep/tests/collections_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http2-api-route-list | epic | - | implemented | passing | conformance | projects/keep/README.md#http-surface-v1 |
| openapi-docs-probes-and-metrics | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| collection-route-breadth | epic | - | implemented | passing | conformance | projects/keep/tests/collections_api.rs |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Surfaces: K8s: `projects/keep/k8s` - StatefulSet, headless/ClusterIP Services, ConfigMap, PDB, and overlays.; Container: `Dockerfile` and `Dockerfile.release` - repo-root and published-binary images.
EC Dimensions: behavior: `cargo test -p keep --test http_api` - readiness, metrics, and graceful drain behavior; stability: `kubectl apply -k projects/keep/k8s/overlays/dev` - deployment dogfood path
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Run Keep as a Kubernetes-native stateful service with PVC-backed instances,
headless and ClusterIP service discovery, probe-aware readiness drain, bounded
config through env/flags, and nonroot container images.
Gate Inventory:
- projects/keep/k8s; projects/keep/Dockerfile; projects/keep/Dockerfile.release; projects/keep/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| statefulset-pvc-service-topology | epic | - | implemented | passing | conformance | projects/keep/k8s |
| readiness-drain-and-probe-contract | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| nonroot-container-image-path | epic | - | implemented | passing | smoke | projects/keep/Dockerfile; projects/keep/Dockerfile.release |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Surfaces: Rust API: `keep::raft` - raftcore-backed state machine for primary/replica convergence.; K8s: StatefulSet - stable network identity and PVC-backed replica pods.; HTTP: future raft network - primary write ownership with replica failover.
EC Dimensions: stability: `cargo test -p keep --test raft_node` - current raft state-machine conformance; behavior: multi-node HTTP/2 raft network - staged failover gate
Root WI: 121
Status: auditing
Required Verification: conformance, dogfood
Promise:
Support a primary/replica HA topology where one write-owning primary is backed
by durable replicas and failover preserves the public KV API. The capability is
tracked separately from the generic HA domain root so the profile explicitly
names the replica requirement.
Gate Inventory:
- projects/keep/tests/raft_node.rs; projects/keep/HA.md; projects/keep/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| single-node-raftcore-state-machine | epic | 121 | implemented | passing | conformance | projects/keep/tests/raft_node.rs |
| primary-replica-topology-contract | epic | 121 | planned | planned | dogfood | projects/keep/HA.md |
| multi-node-replica-failover | epic | 121 | planned | planned | dogfood | projects/keep/HA.md; projects/keep/k8s |

### KV API

ID: kv-api
Type: Runtime
Surfaces: HTTP: `/v1/kv/*`, `/healthz`, `/readyz`, `/metrics`, `/openapi.json` - public service API.; Rust API: `keep::client::KvClient` - in-tree HTTP client.
EC Dimensions: behavior: `cargo test -p keep --test http_api` - public HTTP API conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Expose a cloud-native key-value and claim-check store over HTTP/2 + OpenAPI,
including scalar get/set/delete, batches, scans, locks, probes, metrics, and
opaque blob roundtrips.
Gate Inventory:
- projects/keep/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http-key-value-surface | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |
| claim-check-blob-roundtrip | epic | - | implemented | passing | conformance | projects/keep/tests/http_api.rs |

### Collections

ID: collections
Type: Runtime
Surfaces: HTTP: `/v1/hashes`, `/v1/sets`, `/v1/zsets`, `/v1/lists` - collection APIs.
EC Dimensions: behavior: `cargo test -p keep --test collections_api` - collection operation conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Provide Redis-like hash, set, sorted-set, and list operations on the same
durable engine and HTTP surface.
Gate Inventory:
- projects/keep/tests/collections_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| hash-set-sorted-set-operations | epic | - | implemented | passing | conformance | projects/keep/tests/collections_api.rs |
| list-push-pop-blocking-pop-operations | epic | - | implemented | passing | conformance | projects/keep/tests/collections_api.rs |

### Durability

ID: durability
Type: Runtime
Surfaces: Engine: `KvEngine` + persistence - WAL and snapshot-backed state.; HTTP: mutation APIs - durable-before-ack public writes.
EC Dimensions: stability: `cargo test -p keep --test durability` - cold recovery conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Persist mutations before acknowledgement and recover committed state after a
cold restart.
Gate Inventory:
- projects/keep/tests/durability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| wal-backed-cold-recovery | epic | - | implemented | passing | conformance | projects/keep/tests/durability.rs |

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
| single-node-raftcore-state-machine | epic | 121 | implemented | passing | conformance | projects/keep/tests/raft_node.rs |
| multi-node-http-2-raft-network | epic | 121 | planned | planned | dogfood | projects/keep/HA.md |

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
| relay-keep-worker-facing-contract | epic | 108 | planned | planned | dogfood | pending integration spec |

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
