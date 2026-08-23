# service-k8s status

## Scope

This document describes the current source contract for the reusable
`service-k8s` crate. It does not claim that the named gate ran in this working
session.

Use the [README](README.md) for the public composition workflow. Use the
[roadmap](ROADMAP.md) for future outcomes and explicit non-goals.

## State definitions

| State | Meaning |
|---|---|
| Supported | The current source has a public contract, an implementation, and a named executable gate for the stated scope. |
| Limited | The current source supports the stated scope, but the Limits cell names a material boundary. |
| Not supported | The behavior is not part of the current product contract. The Evidence cell points to a future outcome or a non-goal. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| Managed service reconciliation | `managed-service-reconciliation` | Supported | A namespaced `ManagedService` CR can use the generic watch, leader-election, apply, readiness, Event, metric, prune, and status loop. | The service supplies its CRD, child plan, external admission, and domain status meaning. | `cargo test -p service-k8s` |
| Workload render toolkit | `workload-render-toolkit` | Supported | Callers can compose common Deployment and StatefulSet envelopes, Services, identity, security, disruption, storage, and rollout fields. | The caller owns service-specific topology and values. | `cargo test -p service-k8s` |
| Projected token and RBAC rendering | `projected-token-rbac-rendering` | Supported | Callers can render an audience-bound projected ServiceAccount token plus explicit ClusterRoleBinding, Role, and RoleBinding object shapes. | The caller supplies the audience, mount, subjects, rule semantics, names, ownership labels, and lifecycle. Rendering alone does not grant access until Kubernetes accepts the objects. | `cargo test -p service-k8s` |
| Lifecycle and condition projection | `lifecycle-condition-projection` | Supported | Callers can validate termination budgets, render standard probes, and project condition facts with stable transition times. | The caller decides which facts mean Ready or Degraded for its service. | `cargo test -p service-k8s` |
| Stateful planning and PVC growth | `stateful-planning-pvc-growth` | Supported | Pure helpers plan shard or replica changes. The resize helper can grow scoped PVCs when the StorageClass permits it. | The library does not execute Raft membership. Kubernetes PVC shrink is unsupported. | `cargo test -p service-k8s` |
| Certificate lifecycle | `certificate-lifecycle` | Supported | An injected issuer and Kubernetes store can drive request, rotation, projection, ownership, and status behavior. | The deployment supplies the certificate authority and its authorization. | `cargo test -p service-k8s` |
| Shared Fleet controller | `shared-fleet-controller` | Not supported | No `service_k8s::fleet` module currently owns multi-namespace Fleet watch, adoption, prune, rollout, or status projection. | Lumen keeps its current Fleet materializer in app code. | [Shared Fleet controller](ROADMAP.md#shared-fleet-controller) |
| Declarative access RBAC convergence | `declarative-access-rbac-convergence-surface` | Not supported | No shared controller contract applies, adopts, prunes, and projects status for app-declared access Roles and RoleBindings. | Apps must currently render or apply access resources through their own path. | [Declarative access RBAC convergence](ROADMAP.md#declarative-access-rbac-convergence) |
| Public trust-bundle convergence | `public-trust-bundle-convergence` | Not supported | No shared contract publishes, adopts, rotates, prunes, and projects status for public CA ConfigMaps across client namespaces. | The current certificate mechanism stores scoped certificate material. It does not own app client-trust distribution. | [Public trust-bundle convergence](ROADMAP.md#public-trust-bundle-convergence) |
| Failure-domain placement primitives | `failure-domain-placement-primitives` | Not supported | Current render inputs accept selectors, tolerations, affinity, and topology-spread values, but no shared typed helper scopes hard placement to an app-defined replica group. | Apps currently build their own labels and raw scheduling expressions. | [Failure-domain placement primitives](ROADMAP.md#failure-domain-placement-primitives) |
| Controlled StatefulSet member rollout | `controlled-statefulset-member-rollout` | Not supported | The shared controller does not drive one selected StatefulSet member at a time from app-provided health gates. | PDB rendering alone does not constrain a StatefulSet rolling update. | [Controlled StatefulSet member rollout](ROADMAP.md#controlled-statefulset-member-rollout) |

## Evidence policy

The commands above are required gates for each supported scope. This document
does not store execution output. CI and local test logs own run evidence.

Update a row with any public API, behavior, or evidence change. Move a future
outcome into current support only after the implementation and executable gate
exist.
