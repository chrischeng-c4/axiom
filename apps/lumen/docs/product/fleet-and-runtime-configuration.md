# Fleet and runtime configuration

## Fleet and runtime configuration

- Problem: Fleet instances need safe, explicit runtime configuration and convergence.
- Who: Fleet operators.
- Promise: Fleet materializes declared instances and current override, environment, argument, rollout, readiness, pruning, and readiness-dimension surfaces.
- Status rows: `fleet-instance-materialization`, `fleet-lumen-spec-overrides`, `fleet-extra-environment`, `fleet-extra-arguments`, `fleet-safe-rollout`, `fleet-runtime-readiness-aggregation`, `fleet-child-pruning`, `runtime-readiness-dimensions`.
- Limits today: Fleet convergence and production rollout guarantees are future work.
- Non-goals: Reserved identity, topology, storage, or security overrides.
- Neighbours: Topology owns in-runtime quorum transitions.

## Runtime configuration parity

- Problem: Standalone and Managed configuration can diverge.
- Who: Runtime and Fleet operators.
- Promise: Both paths use one classified configuration contract.
- Outcome: `runtime-configuration-parity`. Tracking: Not assigned.
- Non-goals: Unrestricted raw overrides.
- Open: Classify every option and restart effect.
- Neighbours: Fleet production convergence.

## Fleet production convergence

- Problem: Fleet needs durable, observable convergence across runtimes.
- Who: Fleet operators.
- Promise: Fleet converges declared instances and publishes complete readiness.
- Outcome: `fleet-production-convergence`. Tracking: Not assigned.
- Non-goals: Replacing in-runtime Raft safety.
- Open: Define recovery and aggregate readiness evidence.
- Neighbours: Fleet safe rollout and foundation extraction.

## Non-goals in this area

Fleet does not replace stateful membership safety with generic rollout order.
