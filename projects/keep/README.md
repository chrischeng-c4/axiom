# keep

Cloud-native, multi-core key-value / claim-check store — the loom/relay data
plane and a Redis / Dragonfly replacement. Promoted from `cclab-kv`: the sharded
engine and tiered RAM+disk persistence are unchanged; the transport is now
**HTTP/2 + OpenAPI** (no raw TCP).

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
- ◑ HA — phase A (sharded scale-out + `/cluster`) done; async-replica (B) and raft/quorum (C) planned — see [HA.md](HA.md) (#121)
- ☐ migrate the `ion`-feature TCP consumers (queuekit, queue) off the retired
  `cclab-kv` TCP client; then dedupe the legacy `crates/cclab-kv` +
  `projects/queue/kv` copies.
