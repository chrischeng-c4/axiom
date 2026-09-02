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

## Non-goals in this area

Lumen never places bearer-token values in declarative runtime state.
