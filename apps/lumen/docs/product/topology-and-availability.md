# Topology and availability

## Topology and availability

- Problem: Stateful Lumen needs explicit placement and quorum boundaries.
- Who: Operators of replicated and sharded runtimes.
- Promise: Lumen exposes current capacity, placement, topology, rollout, split, autoscaling, and HPA boundaries.
- Status rows: `capacity-catalog-placement`, `kubernetes-native-placement`, `fixed-topology`, `per-shard-failure-domain-placement`, `quorum-safe-runtime-rollout`, `automatic-shard-splitting`, `membership-aware-replica-autoscaling`, `generic-horizontal-pod-autoscaling`.
- Limits today: Automated replica changes and generic HPA are not supported.
- Non-goals: Claiming one Pod or two voters is highly available.
- Neighbours: Recovery owns restart safety; Fleet owns cross-runtime convergence.

## Per-shard failure-domain placement

- Problem: Replicas need independent failure domains.
- Who: Replicated-runtime operators.
- Promise: Each shard can place members across declared failure domains.
- Outcome: `per-shard-failure-domain-placement`. Tracking: Not assigned.
- Non-goals: Capacity selection outside the catalog.
- Open: Define placement refusal and recovery behavior.
- Neighbours: Kubernetes-native placement.

## Quorum-safe runtime rollout

- Problem: A StatefulSet rollout can remove quorum.
- Who: Replicated-runtime operators.
- Promise: Runtime rollout changes members only when quorum and replication gates allow it.
- Outcome: `quorum-safe-runtime-rollout`. Tracking: Not assigned.
- Non-goals: Treating a PDB as a quorum gate.
- Open: Define member-at-a-time sequencing.
- Neighbours: Bounded shutdown and failover.

## Non-goals in this area

Generic HorizontalPodAutoscaler control is not a Lumen topology contract.
