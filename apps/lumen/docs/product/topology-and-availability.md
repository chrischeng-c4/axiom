# Topology and availability

## Topology and availability

- Problem: Stateful Lumen needs explicit placement and quorum boundaries.
- Who: Operators of replicated and sharded runtimes.
- Promise: Lumen exposes current capacity, placement, topology, rollout, split, autoscaling, and HPA boundaries.
- Status rows: `capacity-catalog-placement`, `kubernetes-native-placement`, `fixed-topology`, `per-shard-failure-domain-placement`, `quorum-safe-runtime-rollout`, `automatic-shard-splitting`, `membership-aware-replica-autoscaling`, `generic-horizontal-pod-autoscaling`.
- Limits today: Automated replica changes and generic HPA are not supported.
- Non-goals: Claiming one Pod or two voters is highly available.
- Neighbours: Recovery owns restart safety; Fleet owns cross-runtime convergence.

## Per-shard failure-domain placement (Milestone #31)

- Problem: Replicas need independent failure domains.
- Who: Replicated-runtime operators.
- Promise: Each shard can place members across declared failure domains.
- Outcome: `per-shard-failure-domain-placement`. Tracking: [Milestone #31](https://github.com/chrischeng-c4/axiom/milestone/31).
- Non-goals: Capacity selection outside the catalog.
- Open: Define placement refusal and recovery behavior.
- Neighbours: Kubernetes-native placement.

## Quorum-safe runtime rollout (Milestone #32)

- Problem: A StatefulSet rollout can remove quorum.
- Who: Replicated-runtime operators.
- Promise: Runtime rollout changes members only when quorum and replication gates allow it.
- Outcome: `quorum-safe-runtime-rollout`. Tracking: [Milestone #32](https://github.com/chrischeng-c4/axiom/milestone/32).
- Non-goals: Treating a PDB as a quorum gate.
- Open: Define member-at-a-time sequencing.
- Neighbours: Bounded shutdown and failover.

## Kubernetes-native placement (Milestone #31)

- Problem: Platform placement needs a portable Kubernetes contract.
- Who: Kubernetes operators.
- Promise: Lumen uses Kubernetes-native placement controls without machine-type fields in the API.
- Outcome: `kubernetes-native-placement`. Tracking: [Milestone #31](https://github.com/chrischeng-c4/axiom/milestone/31).
- Non-goals: Cloud-specific machine types in the CRD.
- Open: Define portable capacity and placement mapping.
- Neighbours: Per-shard failure-domain placement.

## Membership-aware replica autoscaling (Milestone #28)

- Problem: Replica count cannot change safely without Raft membership work.
- Who: Operators under sustained load.
- Promise: Lumen can add or remove replicas only through membership-aware transitions.
- Outcome: `membership-aware-replica-autoscaling`. Tracking: [Milestone #28](https://github.com/chrischeng-c4/axiom/milestone/28).
- Non-goals: Generic HPA control of serving pods.
- Open: Define the safe transition actuator.
- Neighbours: Quorum-safe runtime rollout.

## High-availability shard expansion (Milestone #29)

- Problem: Shard splitting must work with replicated shards.
- Who: High-availability runtime operators.
- Promise: Shard expansion keeps a Raft quorum while it moves ownership.
- Outcome: `high-availability-shard-expansion`. Tracking: [Milestone #29](https://github.com/chrischeng-c4/axiom/milestone/29).
- Non-goals: Unsafe split during replica transition.
- Open: Define restart and rollback evidence.
- Neighbours: Automatic shard splitting.

## Bounded Raft shutdown and failover (Milestone #8)

- Problem: Shutdown and leadership change must finish within explicit bounds.
- Who: Replicated-runtime operators.
- Promise: Raft shutdown and failover expose bounded, recoverable behavior.
- Outcome: `bounded-raft-shutdown-and-failover`. Tracking: [Milestone #8](https://github.com/chrischeng-c4/axiom/milestone/8).
- Non-goals: Unbounded background shutdown.
- Open: Define timer, drain, and retry limits.
- Neighbours: Quorum-safe runtime rollout.

## Deterministic consensus conformance (Milestone #7)

- Problem: Adversarial scheduling can hide consensus safety failures.
- Who: Raft-runtime maintainers and Lumen operators.
- Promise: Deterministic replay proves declared recovery and membership invariants.
- Outcome: `deterministic-consensus-conformance`. Tracking: [Milestone #7](https://github.com/chrischeng-c4/axiom/milestone/7).
- Non-goals: Replacing production network testing.
- Open: Freeze the corpus, replay, and mutant-kill contract.
- Neighbours: Managed embedded data durability.

## Distributed search routing and merge (Milestone #18)

- Problem: A multi-shard search needs safe routing and merge rules.
- Who: Distributed-search callers.
- Promise: Lumen routes search work and merges results through one declared contract.
- Outcome: `distributed-search-routing-and-merge`. Tracking: [Milestone #18](https://github.com/chrischeng-c4/axiom/milestone/18).
- Non-goals: Partial or unordered result claims.
- Open: Define failure, cursor, and merge semantics.
- Neighbours: Distributed facet convergence.

## Non-goals in this area

Generic HorizontalPodAutoscaler control is not a Lumen topology contract.
