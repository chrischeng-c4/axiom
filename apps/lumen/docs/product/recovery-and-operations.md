# Recovery and operations

## Recovery and operations

- Problem: Operators need clear durable-write and PVC lifecycle limits.
- Who: Teams that recover, replace, and retire Managed Lumen.
- Promise: Lumen documents current durable acknowledgement and managed PVC deletion boundaries.
- Status rows: `uniform-durable-write-ack`, `managed-pvc-deletion`.
- Limits today: Uniform durable acknowledgement and regional recovery are future outcomes.
- Non-goals: Broad PVC deletion or recovery of already-lost node-local data.
- Neighbours: Runtime deployment mounts data; topology protects quorum.

## Non-goals in this area

Lumen cannot recover data that a prior node-local runtime already lost.
