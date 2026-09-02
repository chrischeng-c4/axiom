# Runtime and deployment

## Runtime and deployment

- Problem: Operators need supported Standalone and Managed deployment paths.
- Who: Platform teams that deploy Lumen.
- Promise: Lumen supports declared Standalone, GKE rendering, direct Managed, and current activation boundaries.
- Status rows: `standalone-runtime`, `standalone-gke-render`, `standalone-authentication-default`, `direct-kustomize-compatibility`, `gke-zonal-acceptance`, `gke-regional-production-profile`, `direct-managed-instance`, `managed-search-capability-activation`.
- Limits today: Current zonal acceptance does not certify regional production, and Search v2 activation is not supported.
- Non-goals: Standalone high availability.
- Neighbours: Topology owns quorum and placement; recovery owns data retention.

## Non-goals in this area

Standalone deployment is not a high-availability promise.
