# service-k8s

## Brief

`service-k8s` is the shared Kubernetes integration for axiom services: managed
reconciliation, leader election, workload rendering, stateful capacity
planning, and resize primitives.

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
