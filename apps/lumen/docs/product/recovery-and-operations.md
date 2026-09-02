# Recovery and operations

## Recovery and operations

- Problem: Operators need clear durable-write and PVC lifecycle limits.
- Who: Teams that recover, replace, and retire Managed Lumen.
- Promise: Lumen documents current durable acknowledgement and managed PVC deletion boundaries.
- Status rows: `uniform-durable-write-ack`, `managed-pvc-deletion`.
- Limits today: Uniform durable acknowledgement and regional recovery are future outcomes.
- Non-goals: Broad PVC deletion or recovery of already-lost node-local data.
- Neighbours: Runtime deployment mounts data; topology protects quorum.

## Versioned deletes and tombstones

- Problem: A delayed write can resurrect deleted data.
- Who: Versioned-write callers.
- Promise: Versioned deletes retain tombstones that block old writes.
- Outcome: `versioned-deletes-and-tombstones`. Tracking: Not assigned.
- Non-goals: Unbounded tombstone retention.
- Open: Define compaction and retention bounds.
- Neighbours: Idempotent replay and rebuild generations.

## Managed data retention

- Problem: PVC lifecycle needs a deliberate user choice.
- Who: Managed Lumen operators.
- Promise: Managed deletion retains or deletes only the declared instance PVCs.
- Outcome: `managed-data-retention`. Tracking: Not assigned.
- Non-goals: Broad namespace PVC deletion.
- Open: Define finalizer and retry evidence.
- Neighbours: Managed embedded data durability.

## Regional topology migration and backup

- Problem: A regional topology needs safe data migration and backup.
- Who: Regional production operators.
- Promise: Lumen can migrate topology and create recoverable regional backups.
- Outcome: `regional-topology-migration-and-backup`. Tracking: Not assigned.
- Non-goals: Assuming a PVC snapshot alone proves regional recovery.
- Open: Define backup consistency and migration cutover.
- Neighbours: Regional production profile.

## Non-goals in this area

Lumen cannot recover data that a prior node-local runtime already lost.
