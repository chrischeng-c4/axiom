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
      - "The existing compatibility log convergence gate remains the executable proof for late/second node replay until shard-group raft bootstrap receives a dedicated kind gate."
  - id: lumen-topology-existing-backup-seed
    capability_id: replica-sync-bootstrap
    claim_id: external-backup-disaster-recovery-seed
    contract_id: topology-existing-backup-seed
    category: behavior
    command: "cargo test -p lumen --test backup_restore_e2e -- --nocapture"
    assertions:
      - "The backup/restore e2e gate proves cold snapshot restore; issue #1181 owns the production seed-then-raft-catch-up path for empty PVC replicas."
```

## Planned Evidence

| WI | Gap | Required proof |
|---:|---|---|
| 1182 | Versioned virtual-bucket shard map | Multi-shard routing tests covering one large collection and shard-map version coexistence. |
| 1180 | Operator-owned reshard policy | CRD/operator render tests plus kind status evidence for prepare/splitting/catch-up/complete phases. |
| 1181 | Empty-PVC replica bootstrap seed | Seed restore followed by raft delta catch-up, with leader load limits and progress status. |
