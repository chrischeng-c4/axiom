# service-k8s

## Brief

`service-k8s` is the reusable Kubernetes operator toolkit for Axiom services.
A service supplies its CRD type, child-object render plan, readiness targets,
and domain status meaning. The library watches resources, elects one active
controller, applies Kubernetes objects, observes workloads, and projects
status.

The crate also provides workload rendering, projected ServiceAccount tokens,
scoped RBAC objects, lifecycle policy, stateful planning, PVC growth,
controller metrics, and certificate lifecycle mechanisms. It does not define
a service's schema, access policy, topology policy, or health meaning.

## Primary workflow

1. Implement `ManagedService` on a namespaced service CRD.
2. Render the service's desired child objects with the shared helpers.
3. Name the workloads whose ready replica counts matter.
4. Project domain conditions and status from the observed facts.
5. Run `service_k8s::run::<Service>()` in the operator process.

The shared controller watches the CR cluster-wide. Only the leader-election
Lease holder applies changes. A follower keeps watching and can take over.

## Reconcile a managed service

`ManagedService` is the main composition point.

| Hook | Service supplies | Library does next |
|---|---|---|
| `render` | Pure child objects derived from the CR | Server-side applies each object in the CR namespace. |
| `reconcile_plan` | Optional async planning facts and opaque context | Applies children, observes readiness, then returns the context to status projection. |
| `readiness_targets` | Deployment or StatefulSet names | Reads ready replica counts. |
| `status_patch_with_context` | Domain status fields | Patches the status subresource. |
| `conditions` | Clock-free condition facts | Preserves transition times and stamps observed generation. |
| `prunes` | Owned child objects that are no longer desired | Rechecks owner UID before deletion. |

The controller uses server-side apply. The service remains responsible for
valid child identity, domain admission, and any external data used by its plan.

## Compose Kubernetes mechanisms

The public modules are independent mechanisms:

| Module | Use it for |
|---|---|
| `controller`, `service`, `lease`, `metrics` | Watch, reconcile, elect a leader, emit Events, and expose control-plane metrics. |
| `render::common` | Pod composition, labels, owner references, Services, ServiceAccounts, PDBs, resources, and security contexts. |
| `render::deployment` | Deployment envelope and rollout policy without stateful identity. |
| `render::projected_token` | One audience-bound ServiceAccount token volume, mount, and derived file path. |
| `render::rbac` | ClusterRoleBinding, namespaced Role and RoleBinding shapes, explicit subjects, named rules, and wildcard detection. |
| `render` root | StatefulSet compatibility surface, headless Services, PVC templates, shard and ordinal identity, and maintenance jobs. |
| `lifecycle` | Validate termination budgets and render standard probes. |
| `stateful` | Plan shard splitting and whole replica layers without changing membership. |
| `resize` | Compare Kubernetes storage quantities and grow eligible PVCs. It never shrinks. |
| `certificate` | Reconcile certificate requests, rotation, projection, and status through an injected issuer and Kubernetes store. |

There is no `service_k8s::fleet` module today. Fleet ownership, adoption,
rollout, and multi-namespace status remain app code until the
[shared Fleet outcome](ROADMAP.md#shared-fleet-controller) lands.

The certificate module is a current reusable mechanism. It does not make any
app operator own certificates until that app composes an issuer, identities,
Secrets, readiness, and a gate. Public CA ConfigMap convergence,
failure-domain-scoped placement helpers, and controller-driven member rollout
are future shared outcomes.

## Contract discovery

| Need | Source of truth |
|---|---|
| Public Rust API | `cargo doc -p service-k8s --no-deps` |
| Crate surface | `libs/service-k8s/src/lib.rs` |
| Managed service contract | `libs/service-k8s/src/service.rs` |
| Controller order and failure behavior | `libs/service-k8s/src/controller.rs` |
| Workload render inputs | `libs/service-k8s/src/render.rs` and its submodules |
| Executable behavior | `cargo test -p service-k8s` |
| Planned trust, placement, and rollout mechanisms | [ROADMAP.md](ROADMAP.md) |

## Capabilities

Every entry below is an equal library capability. Each source states its direct
contribution.

### Capability index

| Capability | ID | User promise | Sources |
|---|---|---|---|
| Managed reconciliation | `managed-reconciliation` | Reconcile a namespaced service CR into owned Kubernetes children with leader failover and status. | `libs/service-k8s`, `external:kubernetes` |
| Workload rendering | `workload-rendering` | Compose common Deployment and StatefulSet workload contracts without duplicating Kubernetes envelopes. | `libs/service-k8s`, `external:kubernetes` |
| Identity and access rendering | `identity-access-rendering` | Render an audience-bound projected token and scoped RBAC object shapes from app-owned policy inputs. | `libs/service-k8s`, `external:kubernetes` |
| Lifecycle and status projection | `lifecycle-status-projection` | Validate shutdown budgets and publish stable Kubernetes conditions from service facts. | `libs/service-k8s`, `external:kubernetes` |
| Stateful planning and PVC growth | `stateful-planning-pvc-growth` | Plan shard or replica changes and grow eligible PVC objects without hiding unsupported shrink. | `libs/service-k8s`, `external:kubernetes` |
| Certificate lifecycle | `certificate-lifecycle` | Reconcile scoped certificate material, rotation, projection, and status through an injected issuer. | `libs/service-k8s`, `external:kubernetes`, `external:certificate-authority` |

The shared stateful-instance render adapter preserves service-owned identity,
image, storage, and lifecycle. The library does not adopt a runtime or choose
the service's persistence policy.

### Managed reconciliation

- ID: `managed-reconciliation`
- Promise: Watch a service CR, apply its owned children under one field manager,
  observe readiness, publish Events and metrics, and update status through one
  active leader.
- Sources:
  - [`libs/service-k8s`](./) provides `ManagedService`, the generic controller,
    dynamic server-side apply, ownership-checked prune, leader election,
    readiness observation, Events, and controller metrics.
  - `external:kubernetes` stores CRs and children, enforces RBAC and field
    ownership, provides Leases, and reports workload readiness.
- Gate: `cargo test -p service-k8s`

### Workload rendering

- ID: `workload-rendering`
- Promise: Render common workload, Service, identity, storage, security,
  disruption, and rollout envelopes from service-owned inputs.
- Sources:
  - [`libs/service-k8s`](./) provides workload-neutral helpers plus separate
    Deployment and StatefulSet composition surfaces.
  - `external:kubernetes` defines and executes the rendered workload API.
- Gate: `cargo test -p service-k8s`

### Identity and access rendering

- ID: `identity-access-rendering`
- Promise: Render a matching projected-token volume and mount plus explicit
  Role, RoleBinding, and ClusterRoleBinding object shapes from caller inputs.
- Sources:
  - [`libs/service-k8s`](./) provides one-value token projection, explicit
    ServiceAccount subjects, named RBAC rules, owner-safe object shapes, and
    wildcard detection.
  - `external:kubernetes` issues and rotates the projected token, stores the
    RBAC objects, and evaluates their permissions.
- Gate: `cargo test -p service-k8s`

### Lifecycle and status projection

- ID: `lifecycle-status-projection`
- Promise: Reject invalid termination budgets, render standard probes, and
  preserve condition transition times while observed state is unchanged.
- Sources:
  - [`libs/service-k8s`](./) provides lifecycle validation, probe rendering,
    clock-free condition facts, and deterministic condition projection.
  - `external:kubernetes` runs probes and stores the conventional condition
    fields consumed by `kubectl wait` and other controllers.
- Gate: `cargo test -p service-k8s`

### Stateful planning and PVC growth

- ID: `stateful-planning-pvc-growth`
- Promise: Return explicit shard and replica plans and grow only PVCs whose
  current StorageClass permits expansion.
- Sources:
  - [`libs/service-k8s`](./) provides pure planners, quantity parsing, grow,
    no-op, shrink-refusal decisions, scoped listing, and PVC patching.
  - `external:kubernetes` supplies StatefulSet, PVC, and StorageClass semantics
    and enforces expansion support.
- Gate: `cargo test -p service-k8s`

### Certificate lifecycle

- ID: `certificate-lifecycle`
- Promise: Reconcile certificate requests and rotation without coupling a
  service to one certificate authority implementation.
- Sources:
  - [`libs/service-k8s`](./) provides certificate profiles, state transition,
    digest, rotation, projection, status, issuer interface, and Kubernetes
    storage adapter.
  - `external:kubernetes` stores projected certificate material and scoped
    reconciliation state.
  - `external:certificate-authority` verifies requests and issues the signed
    certificate through the injected issuer contract.
- Gate: `cargo test -p service-k8s`

## Supporting documents

| Document | Use it for |
|---|---|
| [STATUS.md](STATUS.md) | Current support boundaries and evidence |
| [ROADMAP.md](ROADMAP.md) | Future shared outcomes and non-goals |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Edit rules and required verification |
