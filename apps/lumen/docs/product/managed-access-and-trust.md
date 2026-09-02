# Managed access and trust

## Managed access and trust

- Problem: Managed Lumen needs a clear identity, authorization, and trust boundary.
- Who: Kubernetes platform and application teams.
- Promise: Managed Lumen supports the current delegated KSA and trust surfaces within their stated limits.
- Status rows: `managed-delegated-ksa-auth`, `human-tokenrequest-access`, `fleet-managed-whole-runtime-access`, `generated-client-automatic-ksa-auth`, `operator-managed-runtime-certificates`, `operator-managed-client-trust`.
- Limits today: The whole-runtime access and rotation contracts are not complete.
- Non-goals: Storing bearer tokens in a CRD or status.
- Neighbours: Client templates project identity; runtime deployment mounts trusted material.

## Managed runtime KSA access

- Problem: A Managed runtime needs explicit whole-runtime access grants.
- Who: Kubernetes ServiceAccount users.
- Promise: Direct Managed and Fleet declarations converge exact runtime access.
- Outcome: `managed-runtime-ksa-access`. Tracking: Not assigned.
- Non-goals: Per-collection authorization.
- Open: Define complete grant replacement and cleanup rules.
- Neighbours: Managed auth unification.

## Projected KSA client auth (Milestone #30)

- Problem: Clients need an explicit, rotation-safe managed credential input.
- Who: In-cluster generated-client users.
- Promise: A Managed KSA profile reads the projected token without exposing it.
- Outcome: `projected-ksa-client-auth`. Tracking: [Milestone #30](https://github.com/chrischeng-c4/axiom/milestone/30).
- Non-goals: Anonymous fallback after authorization failure.
- Open: Define the final projected-token and CA contract.
- Neighbours: Generated-client request resilience.

## Managed auth unification (Milestone #25)

- Problem: Managed paths need one required identity model.
- Who: Managed runtime operators.
- Promise: Managed Lumen uses one required KSA identity and whole-runtime permission model.
- Outcome: `managed-auth-unification`. Tracking: [Milestone #25](https://github.com/chrischeng-c4/axiom/milestone/25).
- Non-goals: Disabled Managed authentication.
- Open: Define migration from current grants.
- Neighbours: Managed runtime KSA access.

## Managed runtime certificates (Milestone #26)

- Problem: Runtime TLS material needs safe operator lifecycle management.
- Who: Managed runtime operators.
- Promise: The operator manages declared runtime certificates and rotation.
- Outcome: `managed-runtime-certificates`. Tracking: [Milestone #26](https://github.com/chrischeng-c4/axiom/milestone/26).
- Non-goals: Logging or embedding private keys.
- Open: Define issuance and rotation integration.
- Neighbours: Managed client trust.

## Managed client trust (Milestone #26)

- Problem: Clients need a managed, rotated trust path.
- Who: Managed client workloads.
- Promise: Client trust material follows the managed runtime certificate contract.
- Outcome: `managed-client-trust`. Tracking: [Milestone #26](https://github.com/chrischeng-c4/axiom/milestone/26).
- Non-goals: Trust-on-first-use.
- Open: Define CA projection and reload behavior.
- Neighbours: Runtime certificates and workload templates.

## Non-goals in this area

Lumen never places bearer-token values in declarative runtime state.
