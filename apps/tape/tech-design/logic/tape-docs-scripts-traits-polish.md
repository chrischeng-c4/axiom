---
id: tape-docs-scripts-traits-polish
summary: >
  Final docs/scripts/traits polish slice for apps/tape (WI #1331, epic
  #1324), closing out the service-archetype convergence after #1325
  service-http, #1326 service-auth, #1327 raft-host, #1328 operator/k8s/
  dockerfile, #1329 backup/spec-gen/clients, and #1330 vat/meter/guard/
  observability. Adds `apps/tape/docs/deployment-handoff.md` and
  `apps/tape/docs/benchmarks-scale.md`, mirroring `projects/lumen/docs/`'s
  layout and depth but scoped to what tape actually ships today: the
  deployment handoff covers the real dockerfile/k8s-operator path (#1328),
  bearer-auth flags (#1326), backup/restore (#1329), and raft HA env
  contract (#1327); the benchmarks page documents the existing
  `apps/tape/tests/tape_perf_gate.rs` local regression gate and
  `apps/tape/tests/tape_vs_nats_jetstream.rs` real-NATS-JetStream replay win
  gate without inventing new benchmark classes. Adds
  `apps/tape/scripts/dev-single.sh` (single-node local dev, embedded
  journal) and `apps/tape/scripts/dev-cluster.sh` (3-node local raft
  cluster using the real `TAPE_DATA_DIR`/`TAPE_PEER_SERVICE`/`TAPE_PEERS`
  plus the standard `REPLICAS_PER_SHARD`/`SHARD_COUNT`/`VOTER_COUNT`/
  `POD_NAME` downward-API quartet consumed by
  `raft_host::ClusterTopology::from_env`), mirroring lumen's
  `scripts/dev-single.sh`/`scripts/dev-cluster.sh` shape. `kind-e2e.sh`/
  `chaos.sh`/`soak.sh` are intentionally NOT added: WI #1328 explicitly
  deferred live kind-cluster proof (no cluster available in that slice,
  per `apps/tape/README.md`'s Kubernetes-Native Deployment section), so a
  kind/chaos/soak script would be untested and aspirational; adding one
  would misrepresent coverage the project does not have. Refreshes
  `apps/tape/README.md`'s Capability Index for internal consistency
  (checking each table row against its section prose below; the domain
  rows Retention And Backfill, Long-Running Stability, and Security
  Hardening keep their existing `planned`/`not_ready` maturity untouched —
  out of scope for this epic). Evaluates whether `apps/tape/aw.toml`'s
  `[capability.profile].traits` should gain `kubernetes_native` now that
  #1328 shipped real CRD/operator/instance render + dockerfile CLI,
  checking `apps/relay/aw.toml` for precedent (relay does NOT carry
  `kubernetes_native` despite shipping render + a kind-failover-smoke
  script, because relay's own Kubernetes-Native Deployment capability row
  stays `not_ready`/dogfood pending live-cluster proof) before deciding.
  No tape Rust source changes; this is docs + scripts + README/aw.toml
  config only, verified by `cargo build -p tape`/`cargo test -p tape`
  staying green (unaffected by doc/script-only changes).
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
(fill)
```
