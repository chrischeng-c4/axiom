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

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Shared Kubernetes Operator Scaffold | - | implemented | verified | smoke | ready | controller, lease, render, resize, and service traits |

### Shared Kubernetes Operator Scaffold

ID: shared-kubernetes-operator-scaffold
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `service_k8s` controller, lease, render, resize, and service modules.
EC Dimensions: behavior: `cargo test -p service-k8s` - operator scaffold contract coverage
Required Verification: smoke
Promise:
Kubernetes-native services can supply their CRD and render model and reuse the
same controller and HA scaffolding.
Gate Inventory: `cargo test -p service-k8s`; libs/service-k8s/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-kubernetes-operator-scaffold-contract | epic | - | implemented | verified | smoke | `cargo test -p service-k8s`; libs/service-k8s/src/lib.rs |
