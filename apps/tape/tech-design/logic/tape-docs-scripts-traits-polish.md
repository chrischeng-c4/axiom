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
---
id: tape-docs-scripts-traits-polish-flow
entry: route
nodes:
  route:
    kind: start
    label: "apps/tape gains docs/deployment-handoff.md, docs/benchmarks-scale.md, scripts/dev-single.sh, scripts/dev-cluster.sh, plus README/aw.toml polish"
  deployment_doc:
    kind: process
    label: "docs/deployment-handoff.md documents the real dockerfile+k8s-operator path (#1328), bearer-auth flags (#1326), backup/restore (#1329), and raft HA env contract (#1327)"
  benchmarks_doc:
    kind: process
    label: "docs/benchmarks-scale.md documents the existing tests/tape_perf_gate.rs local regression gate and tests/tape_vs_nats_jetstream.rs real-NATS-JetStream replay win gate, no new benchmark classes invented"
  dev_single_script:
    kind: process
    label: "scripts/dev-single.sh boots one tape serve process locally with an embedded file-backed journal"
  dev_cluster_script:
    kind: process
    label: "scripts/dev-cluster.sh boots 3 tape serve processes with REPLICAS_PER_SHARD=3/SHARD_COUNT=1/VOTER_COUNT=3/POD_NAME plus TAPE_DATA_DIR/TAPE_PEER_SERVICE/TAPE_PEERS so raft_host::ClusterTopology::from_env resolves peers and the raft group replicates"
  kind_scripts_skipped:
    kind: decision
    label: "kind-e2e.sh/chaos.sh/soak.sh are NOT added: WI #1328 explicitly deferred live kind-cluster proof, so a kind/chaos/soak script would be untested and aspirational"
  readme_refresh:
    kind: process
    label: "README.md Capability Index rows are checked against their section prose for internal consistency and fixed where stale; Retention And Backfill, Long-Running Stability, and Security Hardening maturity is left untouched"
  traits_review:
    kind: process
    label: "aw.toml [capability.profile].traits reviewed against apps/relay/aw.toml precedent for kubernetes_native; only added if the underlying capability is genuinely implemented, not just rendered"
  done:
    kind: terminal
    label: "cargo build -p tape and cargo test -p tape stay green (docs/scripts/config-only change)"
edges:
  - { from: route, to: deployment_doc }
  - { from: route, to: benchmarks_doc }
  - { from: route, to: dev_single_script }
  - { from: route, to: dev_cluster_script }
  - { from: dev_cluster_script, to: kind_scripts_skipped }
  - { from: deployment_doc, to: readme_refresh }
  - { from: benchmarks_doc, to: readme_refresh }
  - { from: kind_scripts_skipped, to: readme_refresh }
  - { from: readme_refresh, to: traits_review }
  - { from: traits_review, to: done }
---
flowchart TD
    route[apps/tape gains docs/scripts + README/aw.toml polish] --> deployment_doc[docs/deployment-handoff.md: dockerfile+operator, auth, backup, raft HA]
    route --> benchmarks_doc[docs/benchmarks-scale.md: existing tape_perf_gate + tape_vs_nats_jetstream gates]
    route --> dev_single_script[scripts/dev-single.sh: single-node local dev]
    route --> dev_cluster_script[scripts/dev-cluster.sh: 3-node local raft cluster]
    dev_cluster_script --> kind_scripts_skipped[kind-e2e/chaos/soak NOT added: #1328 deferred live kind proof]
    deployment_doc --> readme_refresh[README Capability Index internal-consistency pass]
    benchmarks_doc --> readme_refresh
    kind_scripts_skipped --> readme_refresh
    readme_refresh --> traits_review[aw.toml traits reviewed vs relay precedent for kubernetes_native]
    traits_review --> done[cargo build/test -p tape stay green]
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: (fill: spec-id)-verification
requirements:
  example_requirement:
    id: R1
    text: "(fill: requirement text)"
    kind: functional
    risk: medium
    verify: (fill: concrete verification target, e.g. a test name)
---
flowchart TD
    r1[R1 example requirement] --> fill_concrete_verification_target_e_g_a_test_name[(fill: concrete verification target, e.g. a test name)]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/docs/deployment-handoff.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "New docs page (mirrors projects/lumen/docs/deployment-handoff.md): image/binary (dockerfile render + serve), CLI surface, runbooks (binary/docker/k8s kustomize-equivalent operator path #1328), environment variables (TAPE_BIND/STORE/GRACE_SECS/AUTH/TOKEN_REGISTRY_FILE/DATA_DIR/PEER_SERVICE/PEERS #1326 #1327), HTTP surface and probes, smoke sequence, backup/restore runbook (#1329), and release-readiness gates."
  - path: apps/tape/docs/benchmarks-scale.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "New docs page (mirrors projects/lumen/docs/benchmarks-scale.md shape, scoped to tape's actual gates): documents apps/tape/tests/tape_perf_gate.rs (local append/replay/checkpoint regression budget, no external peer win claims) and apps/tape/tests/tape_vs_nats_jetstream.rs (real nats-server -js 20k-event 128-byte-payload backlog replay, Tape zero-copy replay_refs >=1.5x win), how to reproduce both, and the explicit not-yet-calibrated peer list (Kafka/Redpanda/Pulsar/RabbitMQ Streams). No new benchmark classes invented."
  - path: apps/tape/scripts/dev-single.sh
    action: create
    section: logic
    impl_mode: hand-written
    description: "New script (mirrors projects/lumen/scripts/dev-single.sh): single-node local dev, embedded file-backed journal via TAPE_STORE, TAPE_BIND default 127.0.0.1:7137, TAPE_AUTH=off, runs cargo run -p tape --bin tape -- serve."
  - path: apps/tape/scripts/dev-cluster.sh
    action: create
    section: logic
    impl_mode: hand-written
    description: "New script (mirrors projects/lumen/scripts/dev-cluster.sh): 3-node local raft cluster, sets REPLICAS_PER_SHARD=3/SHARD_COUNT=1/VOTER_COUNT=3/POD_NAME plus TAPE_DATA_DIR/TAPE_PEER_SERVICE/TAPE_PEERS so raft_host::ClusterTopology::from_env resolves peers and TapeRaft::from_topology replicates append/checkpoint-put across 3 tape serve processes on distinct ports."
  - path: apps/tape/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Capability Index internal-consistency pass: verify each row's Impl/Verification/Maturity/Production columns match its section prose below, fix any drift found. Retention And Backfill, Long-Running Stability, and Security Hardening maturity claims stay untouched (still planned/not_ready, out of scope for this epic)."
  - path: apps/tape/aw.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "[capability.profile].traits reviewed against apps/relay/aw.toml precedent for kubernetes_native; only added if the Kubernetes-Native Deployment capability is genuinely implemented beyond offline render (checked against README's own no-live-kind-cluster-proof caveat)."
```
