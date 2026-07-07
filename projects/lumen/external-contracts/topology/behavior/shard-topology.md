---
id: lumen-topology-shard-replica-bootstrap-ec
summary: Shard, replica, backup, and bootstrap contract notes for Lumen's Kubernetes-native topology roadmap.
fill_sections: [e2e-test]
---

# EC: Shard, Replica, And Bootstrap Topology

This note keeps the topology contract readable without requiring an agent to
reverse-engineer `raft-host`, the operator renderer, and backup code.

## Contract

- `shardCount` and `replicasPerShard` are independent knobs.
- `totalPods = shardCount * replicasPerShard`.
- `replicasPerShard > 1` enables raft replica semantics inside each shard
  group; `shardCount > 1` alone does not imply raft.
- StatefulSet ordinals map to `shardIndex = ordinal % shardCount` and
  `replicaIndex = ordinal / shardCount`.
- Storage ownership changes are operator workflows. HPA must not change the
  shard map.
- Data routing uses virtual buckets, then a versioned bucket-to-physical-shard
  map. `hash % shardCount` is not the long-term ownership contract.
- Backup/export is a cold DR and bootstrap seed surface. Live replicas
  synchronize through raft log/snapshot mechanics.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-topology-existing-raft-replica-sync
    capability_id: replica-sync-bootstrap
    claim_id: raft-log-replica-sync-existing-pvc
    contract_id: topology-existing-raft-replica-sync
    category: stability
    command: "cargo test -p lumen --test wal_nats_e2e -- --nocapture"
    assertions:
      - "The existing compatibility log convergence gate remains the executable proof for late/second node replay; shard-group topology is now dogfooded by the operator kind profiles."
  - id: lumen-topology-existing-backup-seed
    capability_id: replica-sync-bootstrap
    claim_id: external-backup-disaster-recovery-seed
    contract_id: topology-existing-backup-seed
    category: behavior
    command: "cargo test -p lumen --test backup_restore_e2e -- --nocapture"
    assertions:
      - "The backup/restore e2e gate proves cold snapshot restore; the empty-PVC bootstrap seed path now restores SnapshotV1 before WAL/raft catch-up."
```

## Evidence

| WI | Contract | Proof |
|---:|---|---|
| 1182 | Versioned virtual-bucket shard map | `cargo test -p lumen reshard`; `cargo test -p lumen` covers multi-shard routing for one large collection and versioned map coexistence. |
| 1180 | Operator-owned reshard policy | `cargo test -p lumen --features operator --test operator_render -- --nocapture` covers CRD/render/status topology. |
| 1181 | Empty-PVC replica bootstrap seed | `cargo test -p lumen --bin lumen bootstrap_seed_file_restores_snapshot_before_catchup -- --nocapture` and `cargo test -p lumen` cover seed-before-catch-up. |
| 1179 | Multi-shard and replicated-shard dogfood | `projects/lumen/scripts/kind-e2e.sh` passes with `shardCount=2, replicasPerShard=1` and `shardCount=2, replicasPerShard=3`. |
