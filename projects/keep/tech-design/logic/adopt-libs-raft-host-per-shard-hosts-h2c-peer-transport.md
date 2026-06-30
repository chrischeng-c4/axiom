---
id: keep-raft-host-adoption
summary: >
  Migrate Keep's hand-rolled single-node raft layer onto the shared
  libs/raft-host: a KvEngine RaftStateMachine, one RaftHost per owned shard
  (HashMap<ShardId, RaftHost>) with shard-scoped h2c peer routers merged onto
  the serve app, and write routing through host.propose for sole-applier
  read-your-write.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-raft-host-adoption-contract
entry: build_hosts
nodes:
  build_hosts: { kind: start, label: "ShardHosts::new spawns one RaftHost per cluster.owned_shards() shard" }
  state_machine: { kind: process, label: "KvStateMachine(engine, cluster, shard) implements raft_host::RaftStateMachine" }
  topology: { kind: process, label: "shard_topology derives node_id/membership/peers; single-node = sole voter, HA = per-shard /shard/{id} peer URLs" }
  spawn: { kind: process, label: "RaftHost::spawn(node_id, membership, peers, RaftStore at data_dir/raft/shard-{id}, sm, SnapshotPolicy::EveryEntries)" }
  router: { kind: process, label: "ShardHosts::router nests each host.router() under /shard/{id} so peer Vote/Append/InstallSnapshot ride the h2c serve port" }
  write: { kind: process, label: "ShardHosts::write routes key to cluster.shard_for(key) host and calls host.propose(serde_json(WalOp))" }
  apply: { kind: process, label: "host sole applier decodes WalOp -> RecoveryManager::apply_one(engine); KvStateMachine advances applied_index" }
  ryw: { kind: process, label: "propose returns the applied index (read-your-write); the engine reflects the write" }
  snapshot: { kind: process, label: "snapshot() = shard-filtered dump_values wrapped with up_to; restore() = load_values + applied_index for InstallSnapshot catch-up" }
  stop: { kind: terminal, label: "per-shard groups replicate, fail over, and catch up via the shared host" }
edges:
  - { from: build_hosts, to: state_machine }
  - { from: state_machine, to: topology }
  - { from: topology, to: spawn }
  - { from: spawn, to: router }
  - { from: router, to: write }
  - { from: write, to: apply }
  - { from: apply, to: ryw }
  - { from: ryw, to: snapshot }
  - { from: snapshot, to: stop }
---
flowchart TD
    build_hosts([ShardHosts::new spawns one RaftHost per owned shard]) --> state_machine[KvStateMachine implements raft_host RaftStateMachine]
    state_machine --> topology[shard_topology derives node_id membership peers]
    topology --> spawn[RaftHost::spawn per shard with RaftStore and EveryEntries policy]
    spawn --> router[ShardHosts::router nests host.router under /shard/{id}]
    router --> write[ShardHosts::write routes key to its shard host propose]
    write --> apply[host sole applier decodes WalOp into RecoveryManager::apply_one]
    apply --> ryw[propose returns applied index read-your-write]
    ryw --> snapshot[snapshot shard-filtered dump_values restore load_values]
    snapshot --> stop([per-shard groups replicate fail over and catch up])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-raft-host-adoption-tests
requirements:
  single_node_ryw:
    id: R1
    text: "A single-node-per-shard ShardHosts commits a SET through host.propose and the engine reflects it immediately (read-your-write)."
    kind: behavior
    risk: high
    verify: test
  per_shard_routing:
    id: R2
    text: "ShardHosts routes each write by key to its owning shard host and every committed write lands in the engine."
    kind: behavior
    risk: medium
    verify: test
  shard_snapshot:
    id: R3
    text: "KvStateMachine snapshot/restore round-trips a shard's values via dump_values/load_values, restoring applied_index."
    kind: behavior
    risk: medium
    verify: test
  multi_node_failover:
    id: R4
    text: "A replicated shard group elects a new leader after the leader is SIGKILLed and committed data survives (multi-process cluster; deferred)."
    kind: behavior
    risk: high
    verify: test
  snapshot_catch_up:
    id: R5
    text: "A wiped-data-dir replica of a shard catches up via InstallSnapshot (snapshot_index > 0; multi-process cluster; deferred)."
    kind: behavior
    risk: high
    verify: test
elements:
  raft_node_tests:
    kind: test
    path: projects/keep/tests/raft_node.rs
relations:
  - { from: raft_node_tests, verifies: single_node_ryw }
  - { from: raft_node_tests, verifies: per_shard_routing }
  - { from: raft_node_tests, verifies: shard_snapshot }
  - { from: raft_node_tests, verifies: multi_node_failover }
  - { from: raft_node_tests, verifies: snapshot_catch_up }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "single-node read-your-write"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "per-shard write routing"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "shard snapshot round-trip"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "multi-node failover (deferred)"
      risk: high
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: "snapshot catch-up (deferred)"
      risk: high
      verifymethod: test
    }
    element raft_node_tests {
      type: "rs/#[tokio::test]"
    }
    raft_node_tests - verifies -> R1
    raft_node_tests - verifies -> R2
    raft_node_tests - verifies -> R3
    raft_node_tests - verifies -> R4
    raft_node_tests - verifies -> R5
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the optional raft-host dependency and make the raft feature pull dep:raft-host alongside dep:raft-core."
  - path: projects/keep/src/raft.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Replace RaftKv/ShardedRaft with KvStateMachine (raft_host::RaftStateMachine over KvEngine, shard-filtered snapshot/restore) and ShardHosts (HashMap<ShardId, Arc<RaftHost>>: per-shard spawn, shard_topology derivation, write routing via host.propose, and a router() that nests each host under /shard/{id})."
  - path: projects/keep/src/bin/keep.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "When the raft feature is enabled and replica mode is on, build ShardHosts and merge its shard-scoped peer router onto the serve app so the h2c peer transport rides the serve port."
  - path: projects/keep/tests/raft_node.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Rewrite library tests onto ShardHosts/KvStateMachine: single-node read-your-write, per-shard routing, and shard snapshot round-trip; author ignored multi-process failover and snapshot-catch-up tests."
```
