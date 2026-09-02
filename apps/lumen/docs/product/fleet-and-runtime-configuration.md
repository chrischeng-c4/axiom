# Fleet and runtime configuration

## Fleet and runtime configuration

- Problem: Fleet instances need safe, explicit runtime configuration and convergence.
- Who: Fleet operators.
- Promise: Fleet materializes declared instances and current override, environment, argument, rollout, readiness, pruning, and readiness-dimension surfaces.
- Status rows: `fleet-instance-materialization`, `fleet-lumen-spec-overrides`, `fleet-extra-environment`, `fleet-extra-arguments`, `fleet-safe-rollout`, `fleet-runtime-readiness-aggregation`, `fleet-child-pruning`, `runtime-readiness-dimensions`.
- Limits today: Fleet convergence and production rollout guarantees are future work.
- Non-goals: Reserved identity, topology, storage, or security overrides.
- Neighbours: Topology owns in-runtime quorum transitions.

## Non-goals in this area

Fleet does not replace stateful membership safety with generic rollout order.
