# operator

## Brief

`operator` is the shared Kubernetes operator scaffold for axiom services: a
generic reconcile controller, leader-election lease, and sharded HA render
toolkit.

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
Surfaces: Rust API: `operator` controller, lease, render, resize, and service modules.
EC Dimensions: behavior: `cargo test -p operator` - operator scaffold contract coverage
Required Verification: smoke
Promise:
Kubernetes-native services can supply their CRD and render model and reuse the
same controller and HA scaffolding.
Gate Inventory: `cargo test -p operator`; libs/operator/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-kubernetes-operator-scaffold-contract | epic | - | implemented | verified | smoke | `cargo test -p operator`; libs/operator/src/lib.rs |
