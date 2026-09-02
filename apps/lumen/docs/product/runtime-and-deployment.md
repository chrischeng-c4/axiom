# Runtime and deployment

## Runtime and deployment

- Problem: Operators need supported Standalone and Managed deployment paths.
- Who: Platform teams that deploy Lumen.
- Promise: Lumen supports declared Standalone, GKE rendering, direct Managed, and current activation boundaries.
- Status rows: `standalone-runtime`, `standalone-gke-render`, `standalone-authentication-default`, `direct-kustomize-compatibility`, `gke-zonal-acceptance`, `gke-regional-production-profile`, `direct-managed-instance`, `managed-search-capability-activation`.
- Limits today: Current zonal acceptance does not certify regional production, and Search v2 activation is not supported.
- Non-goals: Standalone high availability.
- Neighbours: Topology owns quorum and placement; recovery owns data retention.

## GKE regional production profile

- Problem: Zonal evidence does not prove a regional production profile.
- Who: GKE production operators.
- Promise: Lumen has an evidence-backed GKE Standard Regional profile.
- Outcome: `gke-regional-production-profile`. Tracking: Not assigned.
- Non-goals: Reusing zonal evidence as regional proof.
- Open: Complete topology, disruption, backup, and recovery drills.
- Neighbours: Regional migration and upgrade recovery.

## GKE Autopilot certification

- Problem: Autopilot has different stateful constraints.
- Who: GKE Autopilot operators.
- Promise: Lumen has a separate evidence-backed Autopilot support tier.
- Outcome: `gke-autopilot-certification`. Tracking: Not assigned.
- Non-goals: Assuming Standard Regional certification applies unchanged.
- Open: Define Autopilot topology and operational evidence.
- Neighbours: GKE regional production profile.

## Managed embedded data durability

- Problem: A single-replica Managed runtime must retain its index and AOF on its PVC.
- Who: Operators of one-replica Managed Lumen.
- Promise: Managed embedded Raft data uses the exact child path on the retained PVC.
- Outcome: `managed-embedded-data-durability`. Tracking: Not assigned.
- Non-goals: Recovering data already lost from node-local storage.
- Open: Verify restart and legacy-PVC adoption through the release oracle.
- Neighbours: Managed data retention and deterministic consensus conformance.

## Non-goals in this area

Standalone deployment is not a high-availability promise.
