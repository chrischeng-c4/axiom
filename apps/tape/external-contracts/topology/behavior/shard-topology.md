<!-- HANDWRITE-BEGIN gap="missing-generator:logic:ffe3dec3" tracker="pending-tracker" reason="Tape shard/replica, durable replay, backup seed, and operator topology contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)." -->
---
id: tape-topology-shard-replica-bootstrap-ec
summary: Shard, replica, snapshot, and bootstrap topology contract for Tape's replay journal.
fill_sections: [e2e-test]
---

# EC: Shard, Replica, And Bootstrap Topology

Lumen's topology EC is the reference shape. Tape's external service contract
remains Topic to N subscriptions; shards and replicas are internal storage and
availability topology, not a Kafka-compatible public partition abstraction.

## Contract

- `SHARD_COUNT` and `REPLICAS_PER_SHARD` describe independent storage and
  availability axes in the shared StatefulSet projection.
- `REPLICAS_PER_SHARD > 1` activates the shared Raft leader/follower path for a
  shard group; a local single-node Tape instance needs no service-specific
  Raft flag.
- Topic replay, offsets, and consumer checkpoints remain Tape domain state;
  the topology layer does not expose a second public partition model.
- Backup snapshots seed cold recovery and empty-PVC bootstrap. Live replicas
  converge through Raft log and InstallSnapshot mechanics.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-topology-existing-raft-replica-sync
    capability_id: primary-replicas
    claim_id: tape-raft-log-replica-sync-existing-pvc
    contract_id: tape-topology-existing-raft-replica-sync
    category: stability
    command: "cargo test -p tape --test raft_cluster --test raft_failover --test raft_persistence -- --test-threads=1"
    assertions:
      - "A Tape Raft group elects, replicates committed journal appends, forwards follower appends to its leader, and retains the durable applied floor across restart."
      - "A fresh Tape node catches up by InstallSnapshot and a killed leader is replaced by a surviving elected group without committed-event loss."
  - id: tape-topology-existing-backup-seed
    capability_id: primary-replicas
    claim_id: tape-external-backup-disaster-recovery-seed
    contract_id: tape-topology-existing-backup-seed
    category: behavior
    command: "cargo test -p tape --test bootstrap -- --nocapture"
    assertions:
      - "Tape's empty-PVC bootstrap seed restores a consistent journal snapshot before normal Raft catch-up."
      - "The backup seed is cold recovery evidence and does not replace live Raft replication."
```
<!-- HANDWRITE-END -->
