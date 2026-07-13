---
id: '1575'
summary: Reconcile a typed Pgpool custom resource into stateless shared operator artifacts and project readiness plus global endpoint connection-budget status.
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    CR[Pgpool custom resource] --> MS[ManagedService reconcile]
    MS --> DEP[Stateless Deployment]
    MS --> SVC[ClusterIP Service]
    MS --> PDB[PodDisruptionBudget]
    DB[PostgreSQL endpoint facts] --> CP[PgpoolControlPlane]
    CP --> QUOTA[Global endpoint quota]
    QUOTA --> POD[Per-Pod backend limit]
    CP --> STATUS[Pgpool status and metrics]
    DEP --> READY[Deployment ready replicas]
    READY --> STATUS
```
