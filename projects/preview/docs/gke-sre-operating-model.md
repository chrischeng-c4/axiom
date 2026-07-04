# GKE SRE Operating Model

`preview` starts as a CI-driven binary because a new team cluster may not allow
CRD installation, controller service accounts, or custom ingress extensions on
day one. The binary renders the same desired-state model a later controller
would reconcile.

## Ownership

The SRE-owned contract is:

- MR identity: source MR id, commit SHA, image digest/tag, owner, TTL.
- Base projection: stable namespace such as `uat-base` supplies the workload
  contract to copy from.
- Runtime projection: namespace `uat-mr-<id>` plus workload labels and probes.
- Route projection: target `mr-<id>` mapped to namespace/service/port.
- Lifecycle: create/update on MR changes, drain/delete on merge or close,
  retain failed previews for a bounded debug TTL.

Application teams own app readiness, migrations, and test data behavior.
Platform/SRE owns route safety, namespace cleanup, RBAC, quota, and audit.

## Routing Boundary

Do not expose namespaces to testers. Testers select a stable target:

- browser: signed cookie `uat_target=mr-123`
- API/mobile/manual client: header `X-UAT-Target: mr-123`

The router resolves the target through a route binding. The first renderer emits
that binding as a ConfigMap in a control namespace so the adapter can be a small
router deployment, an ingress controller extension, or a GitOps-rendered input.

## GKE Defaults

First useful defaults for an SRE lead:

- one namespace per MR;
- a base namespace such as `uat-base` is the stable source workload namespace;
- namespace labels for MR, SHA, owner, and app;
- workload clone plans copy template intent from base while excluding runtime
  identity, base secrets by default, and allocated service fields;
- service account name rendered explicitly for Workload Identity review;
- namespace-local quota, limit defaults, and read-only workload RBAC;
- `/readyz` and `/healthz` probes required by the rendered Deployment;
- no direct DB clone assumption in the preview project yet;
- all delete behavior starts as a dry-run cleanup plan and protects the base
  namespace plus control namespace.

## Adapter Path

Use the same model through progressively stronger deployment forms:

1. CI binary renders YAML, prints an ordered apply plan, and comments on the MR.
2. CI binary applies to kind/sandbox clusters directly or renders into a GitOps
   repo for ArgoCD/Flux.
3. A preview-router watches route-binding ConfigMaps and routes cookie/header
   traffic.
4. When the cluster policy allows it, promote `PreviewEnvironment` to a CRD and
   run the same reconcile logic as an operator.

## EC Strategy

The EC path is deliberately layered:

1. Always-on render EC validates deterministic output, manifest inventory, and
   naming.
2. Always-on router EC validates cookie/header target resolution without a live
   ingress.
3. Always-on Kubernetes object EC parses rendered manifests and checks
   cross-resource references.
4. Always-on local CI/CD EC validates `preview apply --plan-only` and
   `preview gitops render` without a live cluster.
5. Opt-in kind/GKE EC builds a local probe image, creates a local base namespace
   fixture, discovers it, applies the rendered objects through `preview apply`,
   runs server-side dry-run after namespace creation, re-applies idempotently,
   waits for rollout, checks service endpoints, reaches `/readyz` through
   port-forward, validates least-privilege RBAC, verifies quota rejection, and
   cleans only test-created namespaces when `PREVIEW_KIND_E2E=1` is set.

This keeps daily CI cheap while making the cluster gate concrete enough for an
SRE-owned validation lane.

## Non-Goals For The Bootstrap

- No production backup restore or DB clone behavior.
- No assumption about Istio, NGINX, Traefik, or GKE Gateway internals.
- No in-cluster controller or CRD install requirement for the first CLI path.
- No shared UAT database migration execution.
