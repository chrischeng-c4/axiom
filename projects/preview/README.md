# preview

## Brief

`preview` manages MR-scoped UAT preview environments for GKE. It is built for
teams that need several features tested against the same UAT entrypoint without
teams overwriting one shared namespace.

The first implementation is intentionally a CLI/reconciler binary, not a
cluster-installed CRD. It renders the contract an SRE lead can review and wire
into CI/CD: one namespace per MR derived from a base UAT workload, a stable
route target, a router-consumable binding, MR comment text, and cleanup plans.
The same internal model can later run as a Kubernetes controller when the
target cluster's CRD/RBAC/GitOps policy is known.

## Capabilities

Canonical field-style capability contracts below are machine-readable input for
`aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| GKE UAT Preview Environment Rendering | - | implemented | verified | smoke | ready | `preview discover-base` normalizes a base Deployment/Service contract; `preview render` emits a base-workload clone plan plus optional fake-GCP data plan/Secret, namespace, service account, quota, limits, RBAC, deployment, service, route binding, MR comment, manifest inventory, and cleanup plan files for an MR-scoped UAT preview. |
| Preview External Contracts | - | implemented | verified | smoke | ready | Always-on render/router-adapter/Kubernetes object/local apply and GitOps tests plus an opt-in kind/GKE-like lifecycle gate that applies, dry-runs, re-applies, loads route-binding ConfigMaps, rolls out, routes, and cleans a preview namespace. |
| Kubernetes-Native Deployment | - | planned | planned | none | not_ready | future CRD/operator form for reconciling the same PreviewEnvironment model in GKE. |

### GKE UAT Preview Environment Rendering

ID: gke-uat-preview-environment-rendering
Type: Devops
Surfaces: CLI: `preview discover-base`, `preview render`, `preview data plan`, `preview data apply`, `preview data cleanup`, `preview apply`, `preview gitops render`, `preview router resolve`, `preview cleanup plan`, `preview cleanup apply`, `preview comment`, `preview cleanup-plan`, `preview llm`, `preview upgrade`, `preview issue`.
EC Dimensions: behavior: render/discovery contract tests - base workload normalization, MR identity, namespace naming, GKE labels, route binding stability, MR comment text, and cleanup dry-run output.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
`preview` turns an MR id, commit SHA, image, and base namespace into a
GKE-oriented UAT preview contract without requiring cluster-specific CRDs up
front. When a base namespace is available locally, `preview discover-base`
reads its Deployment/Service contract and `preview render --base-contract`
embeds the discovered shape into the clone plan. The rendered output can then
be inspected through `preview apply --plan-only`, applied or server-side
dry-run through `preview apply`, or packaged into a relative-path GitOps bundle
with `preview gitops render`. When data flags are supplied, Preview also emits
a local fake-GCP data lifecycle plan and a namespace-local Secret reference so
teams can prove Cloud SQL/AlloyDB-style clone wiring before paying for real GCP
resources.
Gate Inventory:
- `cargo test -p preview`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Base workload discovery | change | #1108 | implemented | verified | smoke | `cargo test -p preview --test base_discovery_contract`; `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture` |
| Local apply and GitOps execution | change | #1109 | implemented | verified | smoke | `cargo test -p preview --test local_cicd_contract`; `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture` |
| Guarded cleanup janitor | change | #1111 | implemented | verified | smoke | `cargo test -p preview --test local_cicd_contract local_cleanup_janitor_plan_reports_guarded_actions`; `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture` |
| Local fake-GCP data lifecycle | change | - | implemented | verified | smoke | `cargo test -p preview --test local_cicd_contract local_data_plan_fake_provider_and_secret_rewrite_are_deterministic` |
| MR-scoped namespace projection | epic | - | implemented | verified | smoke | `cargo test -p preview render_creates_gke_contract_files` |
| Cookie/header route binding contract | epic | - | implemented | verified | smoke | `cargo test -p preview route_binding_uses_target_not_namespace_cookie` |
| Cleanup dry-run planning | epic | - | implemented | verified | smoke | `cargo test -p preview cleanup_plan_marks_closed_mr_for_namespace_delete` |

### Preview External Contracts

ID: preview-external-contracts
Type: Devops
Surfaces: Test gates: `cargo test -p preview`; opt-in kind gate: `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture`.
EC Dimensions: behavior: render contract and router contract; stability: cleanup contract and Kubernetes object cross-reference checks; deployment: opt-in kind/GKE apply, rollout, port-forward, RBAC, quota, and cleanup smoke.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Preview EC is layered so daily development does not require a real cluster, but
the project still has a concrete path to cluster validation.
Gate Inventory:
- `cargo test -p preview`
- `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Render contract EC | epic | - | implemented | verified | smoke | `cargo test -p preview --test render_contract` |
| Local apply/GitOps execution EC | change | #1109 | implemented | verified | smoke | `cargo test -p preview --test local_cicd_contract`; `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture` |
| Local router adapter | change | #1110 | implemented | verified | smoke | `cargo test -p preview --test router_contract`; `cargo test -p preview --test local_cicd_contract local_router_resolve_proves_base_preview_and_fail_closed`; `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture` |
| CI template lifecycle | change | #1112 | implemented | verified | smoke | `cargo test -p preview --test local_cicd_contract ci_templates_document_required_variables_and_command_order`; `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture` |
| Router target EC | epic | - | implemented | verified | smoke | `cargo test -p preview --test router_contract` |
| Kubernetes object EC | epic | - | implemented | verified | smoke | `cargo test -p preview --test k8s_object_contract` |
| Kind/GKE lifecycle EC | epic | - | implemented | verified | smoke | `PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture` |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Surfaces: K8s: future `PreviewEnvironment` CRD/operator; current bootstrap renders Kubernetes Namespace, ServiceAccount, ResourceQuota, LimitRange, Role, RoleBinding, Deployment, Service, and route-binding ConfigMap artifacts.
EC Dimensions: behavior: pending operator/CRD gate - CRD schema, RBAC, controller reconcile, route binding watch, namespace lifecycle, and GKE dry-run.
Root WI: -
Status: confirmed
Required Verification: smoke, conformance
Promise:
Preview can promote its CI-rendered `PreviewEnvironment` model into a
Kubernetes-native controller when the target GKE cluster's CRD/RBAC/GitOps
policy is known.
Gate Inventory:
- pending: projects/preview/tests/kind_lifecycle.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| preview-environment-crd-and-controller | epic | - | planned | planned | none | pending operator/CRD implementation |

## Model

Preview state is modeled as:

```text
Base Namespace -> Base Workload -> PreviewEnvironment -> Preview Namespace -> RouteBinding -> CleanupPlan
                                      \-> DataPlan -> Preview DB Secret -> Fake Provider State
```

The route target is stable (`mr-123`) and intentionally separate from the
namespace (`uat-mr-123`). A cookie or header chooses the route target; the
router resolves that target through a binding. This keeps testers away from
namespace names and lets SREs rebuild or drain runtime resources without
changing shared UAT links.

The base namespace, such as `uat-base`, is the source of truth for the stable
UAT workload shape. Preview copies only the workload contract it needs into
`uat-mr-<id>` and overrides image/SHA/route identity. Runtime identity fields
such as `uid`, `resourceVersion`, `clusterIP`, owner references, and base
namespace secrets are intentionally excluded. Cleanup may delete preview
namespaces and route bindings, but the base namespace and control namespace are
protected namespaces.

Data lifecycle is modeled separately from Kubernetes workload cloning. For the
local-first path, `preview data plan` and `preview render --data-*` describe a
Cloud SQL/AlloyDB-like preview database target, render a `DATABASE_URL` Secret
inside `uat-mr-<id>`, and record fake provider ownership in a local JSON state
file. The fake provider proves naming, TTL, Secret rewrite, idempotent apply,
and guarded cleanup without contacting GCP.

## GKE Assumptions

The first renderer assumes:

- one Kubernetes namespace per MR;
- a stable base namespace such as `uat-base` provides the source workload
  contract;
- standard Kubernetes `Namespace`, `ServiceAccount`, `ResourceQuota`,
  `LimitRange`, `Role`, `RoleBinding`, `Deployment`, and `Service` resources;
- a preview router, gateway extension, or ingress adapter consumes
  `preview.cclab.dev/route-binding` ConfigMaps;
- optional data lifecycle artifacts can model Cloud SQL/AlloyDB clone or
  restore intent without calling Google APIs;
- browser traffic selects a preview with signed cookie `uat_target=mr-123`;
- API/mobile/manual clients can select the same preview with
  `X-UAT-Target: mr-123`;
- real Cloud SQL/AlloyDB clone/restore calls are deferred to the provider
  adapter pilot.

## CLI

| Verb | Purpose |
|---|---|
| `preview discover-base` | Read a base namespace Deployment/Service through `kubectl` and emit a normalized workload contract. |
| `preview render` | Render the MR-scoped preview contract to files. |
| `preview data plan` | Render a local-first data lifecycle plan for fake GCP Cloud SQL-style preview DB wiring. |
| `preview data apply` | Apply a fake provider data plan to a local JSON state file. |
| `preview data cleanup` | Remove fake provider data resources from the local JSON state file with preview ownership guardrails. |
| `preview apply` | Print an ordered plan, server-side dry-run, or apply rendered manifests through `kubectl` with kind-context guardrails. |
| `preview gitops render` | Convert rendered manifests into a deterministic relative-path GitOps bundle. |
| `preview router resolve` | Load rendered route-binding files or kind ConfigMaps and return a base/preview/not-found routing decision. |
| `preview cleanup plan` | Compute guarded keep/drain/delete cleanup decisions from MR, TTL, namespace, route-binding, and protected namespace state. |
| `preview cleanup apply` | Apply a guarded janitor plan through `kubectl`, deleting only bounded preview namespaces and route-binding ConfigMaps. |
| `preview comment` | Print the MR comment text for a rendered preview. |
| `preview cleanup-plan` | Print a dry-run cleanup decision for a preview. |
| `preview llm` | Print offline agent-facing usage notes. |
| `preview upgrade` | Placeholder for the repo-wide self-update convention. |
| `preview issue` | Placeholder for the repo-wide issue convention. |

## Install

Install the release binary for CI runners or SRE laptops:

```bash
curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/preview/install.sh | sh
```

The installer follows the repo-wide binary convention:

- `PREVIEW_VERSION` selects a `preview@*` release tag, defaulting to latest.
- `PREVIEW_INSTALL` selects the install directory, defaulting to `$HOME/.local/bin`.
- `PREVIEW_REPO` selects the GitHub repository, defaulting to `chrischeng-c4/axiom`.
- `GH_TOKEN` or `GITHUB_TOKEN` is used for private forks, with `gh auth token` fallback.

## CI/CD Templates

Copyable local-first lifecycle templates live in
`projects/preview/docs/ci-templates/`:

- `github-actions-preview.yaml` for pull request open/update/rerun/close.
- `gitlab-ci-preview.yml` for GitLab merge request pipelines.
- `local-kind-lifecycle.sh` for SRE laptop smoke testing.

The templates parameterize registry, Kubernetes context, base namespace, app,
host, and TTL so GKE-specific values can be swapped in during the pilot without
changing the command lifecycle.

## External Contract Gates

Preview EC has four layers:

- Render EC: validates the file contract and MR-to-namespace naming.
- Local CI/CD EC: runs the `preview` binary locally to simulate MR
  open/update/comment/close, apply plan summaries, and GitOps bundle rendering
  without a live cluster. It also validates the copyable CI/CD templates.
- Router EC: validates cookie/header target resolution without a real ingress.
- Local router adapter EC: validates base fallback, header/cookie preview
  target routing, and invalid-target fail-closed decisions through
  `preview router resolve`.
- Kubernetes object EC: parses rendered manifests and checks cross-object
  references such as Service selectors, resource bounds, and workload RBAC.
- Kind/GKE lifecycle EC: opt-in SRE gate that requires Docker, `kind`, and
  `kubectl`, then builds a local probe image, creates a local base namespace,
  applies the rendered objects twice, waits for rollout, checks endpoints,
  reaches `/readyz` through port-forward, validates least-privilege RBAC,
  verifies quota rejection, and cleans only test-created namespaces.

## Local-First Validation

Run the no-cluster lane on every change:

```bash
cargo test -p preview
```

That covers render determinism, base workload normalization from Kubernetes
JSON fixtures, clone-plan shape, local CI command shape, MR comments, cleanup
JSON protected namespaces, guarded cleanup janitor decisions, route resolution, local router adapter decisions,
Kubernetes object
cross-references, RBAC shape, and resource bounds.

Run the local-cluster lane before involving a real GKE cluster:

```bash
PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture
```

That covers Docker image build/load, base namespace fixture discovery through
  `preview discover-base`, `preview apply` direct apply/server dry-run/reapply,
  route-binding ConfigMap table loading, rollout, Service
endpoints, port-forward HTTP, admission rejection for oversized pods,
least-privilege RBAC, guarded janitor cleanup apply, and namespace cleanup.
