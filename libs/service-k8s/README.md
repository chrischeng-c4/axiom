# service-k8s

## Brief

`service-k8s` is the shared Kubernetes integration for axiom services: managed
reconciliation, leader election, workload rendering, stateful capacity
planning, and resize primitives.

The render surface is split by mechanism and workload profile:

- `render::common` owns workload-neutral Pod composition plus ordinary
  ServiceAccount, ClusterIP Service, PDB, labels, owner references, and
  resource helpers.
- The existing render root remains the StatefulSet compatibility surface for
  stable identity, headless Services, PVCs, shard/ordinal topology, resize,
  and reshard consumers.
- `render::deployment` owns only the `apps/v1` Deployment envelope and rollout
  policy. It adds no PVC, headless Service, session affinity, shard, ordinal,
  Raft, or stable Pod identity contract.

`ManagedService::reconcile_plan(Client)` is optional. Its default wraps the
existing pure `render()` result with null context, so existing services retain
their behavior. Services with external admission constraints may return an
opaque context; the shared controller follows `plan -> apply children ->
readiness -> status_patch_with_context`. Provider discovery, quotas, and
domain-specific status remain owned by the service.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Shared Kubernetes Operator Scaffold | - | controller, lease, render, resize, and service traits |

### Shared Kubernetes Operator Scaffold

Kubernetes-native services can supply their CRD and render model and reuse the
same controller and HA scaffolding.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `service_k8s` controller, lease, render, resize, and
  service modules.
- Gate — behavior: `cargo test -p service-k8s` - operator scaffold contract
  coverage
- Gate: `cargo test -p service-k8s`
- Source: `libs/service-k8s/src/lib.rs`
- Evidence: `cargo test -p service-k8s`; libs/service-k8s/src/lib.rs
