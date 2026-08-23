# service-k8s roadmap

## Purpose

This document records future shared-library outcomes and explicit non-goals. It
does not describe current support. [STATUS.md](STATUS.md) owns that contract.

The issue tracker owns assignees, work state, schedules, and delivery history.
Stable IDs here let current limits point to one destination.

## Near-term outcomes

### Shared Fleet controller

- ID: `shared-fleet-controller`
- Outcome: Services can compose a watch-driven multi-namespace Fleet controller
  from reusable ownership, adoption, prune, rollout, and status-projection
  mechanics.
- Boundary: The library owns generic types and convergence mechanics. Each app
  supplies child identity, seed and steady-state objects, protected paths, and
  health mapping. The first consumer preserves Lumen's existing materialization
  behavior before it adds readiness and rollout behavior.
- Completion evidence: Library tests prove duplicate-target rejection,
  ownership and adoption safety, Retain and Delete prune behavior, Fleet and
  child watches, low-frequency repair, per-entry API failure isolation, status
  projection, ordered concurrency, and leader failover. Lumen adapter tests
  prove parity for current child objects and protected reshard fields.
- Tracking: Not assigned.

### Declarative access RBAC convergence

- ID: `declarative-access-rbac-convergence`
- Outcome: Apps can reconcile a typed set of namespaced ServiceAccount access
  grants through reusable Role, RoleBinding, ownership, prune, and status
  mechanisms.
- Boundary: The library owns generic object identity, explicit ServiceAccount
  subjects, apply, adoption, prune, and status facts. The app owns its access
  CRD, audience, API group, resource, resource name, verb, readiness meaning,
  and replacement rules. The library does not create namespaces,
  ServiceAccounts, client workloads, or token Secrets.
- Completion evidence: Library tests prove cross-namespace ServiceAccount
  subjects, exact named rules, explicit deny-all as an empty desired grant set,
  create and update, ownership refusal, adoption, stale RoleBinding prune,
  per-object failure isolation, status projection, wildcard refusal, and no
  Secret rendering. App adapter tests prove their resource mapping and policy
  semantics without adding them to the shared library.
- Tracking: Not assigned.

### Public trust-bundle convergence

- ID: `public-trust-bundle-convergence`
- Outcome: Apps can publish public CA bundles into declared namespaces through
  reusable ConfigMap ownership, adoption, rotation-overlap, prune, and status
  mechanics.
- Boundary: The library handles public bytes and generic object convergence.
  The app supplies runtime identity, target namespaces, ConfigMap name and
  labels, old-and-new root overlap, readiness meaning, and retention policy.
  The library does not create namespaces, ServiceAccounts, client Deployments,
  token Secrets, certificate identities, or CA policy.
- Completion evidence: Library tests prove multi-namespace create and update,
  shared desired objects, exact owner identity, adoption refusal, old-and-new
  root overlap, stale owned-object prune, foreign-object retention, per-object
  failure isolation, status facts, public-data-only validation, and leader
  failover.
- Tracking: Not assigned.

### Failure-domain placement primitives

- ID: `failure-domain-placement-primitives`
- Outcome: Apps can compose typed selectors, tolerations, topology intent, and
  group-scoped hard anti-affinity without writing raw Kubernetes expressions.
- Boundary: The app supplies the group identity, such as one shard, and decides
  whether host or zone separation is required. The library validates label
  selectors and topology keys, then renders standard Kubernetes scheduling
  fields. It does not know shards, quorum, GKE machine types, ComputeClasses,
  or node-pool lifecycle.
- Completion evidence: Pure render tests prove group-scoped host anti-affinity,
  required and preferred zone spread, selectors, tolerations, deterministic
  labels, allowed node sharing by different groups, malformed-input refusal,
  and no cloud-specific field in the shared contract.
- Tracking: Not assigned.

### Controlled StatefulSet member rollout

- ID: `controlled-statefulset-member-rollout`
- Outcome: An app can drive one selected StatefulSet member update at a time and
  provide the health facts that permit the next member.
- Boundary: The library owns partition and member selection, desired and
  observed generation tracking, pause and resume, reconciliation recovery, and
  status facts. The app owns group membership, quorum, replication readiness,
  searchable readiness, ordering, and failure meaning. A PDB remains an
  eviction control. The rollout mechanism does not treat it as a StatefulSet
  update gate.
- Completion evidence: Library fixtures prove one-at-a-time selection,
  generation gating, app-health gating, pause and resume, leader failover,
  interrupted reconcile recovery, no second member while the first is blocked,
  and explicit separation between voluntary-eviction policy and workload
  rollout.
- Tracking: Not assigned.

## Later outcomes

No items.

## Non-goals

### Service-specific topology policy

- ID: `service-specific-topology-policy`
- Reason: Each service owns the meaning and safe transition rules for its
  shards, replicas, membership, storage, and health. The library only composes
  typed mechanisms and app-provided projections.

### Namespace and cluster provisioning

- ID: `namespace-and-cluster-provisioning`
- Reason: Cluster and namespace lifecycle includes platform policy, quota,
  identity, and ownership. Fleet mechanics operate inside supplied scopes.
