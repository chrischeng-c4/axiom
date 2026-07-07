---
id: preview-local-apply-gitops-execution
summary: >
  Add operator-facing local execution surfaces for Preview. The CLI reads a
  rendered preview directory, produces a deterministic manifest inventory,
  supports plan-only output, runs guarded kubectl apply/server dry-run for kind
  clusters, and renders a relative-path GitOps bundle for PR-based delivery.
capability_refs:
  - id: "gke-uat-preview-environment-rendering"
    role: primary
    gap: "local-apply-and-gitops-execution"
    claim: "local-apply-and-gitops-execution"
    coverage: partial
    rationale: >
      Work item #1109 turns rendered Preview manifests into executable local
      apply and GitOps bundle paths before real GKE validation.
  - id: "preview-external-contracts"
    role: primary
    gap: "local-apply-gitops-execution-ec"
    claim: "local-apply-gitops-execution-ec"
    coverage: partial
    rationale: >
      The same work item adds local CI/CD and kind EC coverage for apply and
      GitOps execution.
fill_sections: [logic, schema, cli, unit-test, e2e-test, changes]
---

# TD: Preview Local Apply And GitOps Execution

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: preview-local-apply-gitops-flow
entry: preview_apply_or_gitops
nodes:
  rendered_dir: { kind: start, label: "rendered preview dir" }
  inventory: { kind: process, label: "read manifests in deterministic order" }
  guardrails: { kind: decision, label: "protected namespace and kind-context guardrails" }
  plan_only: { kind: terminal, label: "print MR-friendly ordered apply summary" }
  kubectl_apply: { kind: process, label: "kubectl apply or --dry-run=server" }
  gitops: { kind: process, label: "copy ordered manifests to relative-path bundle" }
  done: { kind: terminal, label: "summary or GitOps bundle written" }
edges:
  - { from: rendered_dir, to: inventory }
  - { from: inventory, to: guardrails }
  - { from: guardrails, to: plan_only }
  - { from: guardrails, to: kubectl_apply }
  - { from: guardrails, to: gitops }
  - { from: kubectl_apply, to: done }
  - { from: gitops, to: done }
---
flowchart TD
    rendered_dir([rendered preview dir]) --> inventory[Manifest inventory in fixed order]
    inventory --> guardrails{Guardrails}
    guardrails --> plan_only[Plan-only summary]
    guardrails --> kubectl_apply[kubectl apply / server dry-run]
    guardrails --> gitops[GitOps bundle render]
    kubectl_apply --> done([summary printed])
    gitops --> done
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
types:
  ManifestInventory:
    fields:
      schemaVersion: u8
      namespace: string
      routeTarget: string
      entries: list<ManifestInventoryEntry>
  ManifestInventoryEntry:
    fields:
      order: usize
      path: string
      apiVersion: string
      kind: string
      namespace: string?
      name: string
  ApplyOptions:
    fields:
      dir: path
      context: string?
      dryRun: bool
      allowNonKind: bool
      planOnly: bool
order:
  - k8s/namespace.yaml
  - k8s/service-account.yaml
  - k8s/resource-quota.yaml
  - k8s/limit-range.yaml
  - k8s/workload-role.yaml
  - k8s/workload-role-binding.yaml
  - k8s/deployment.yaml
  - k8s/service.yaml
  - router/route-binding.yaml
guardrails:
  - "manifest paths must be relative and bounded"
  - "Namespace objects cannot target protected namespace names"
  - "actual apply refuses non-kind contexts unless --allow-non-kind is explicit"
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: preview apply
    args:
      --dir: "rendered preview directory"
      --context: "optional kubectl context"
      --dry-run: "use kubectl apply --dry-run=server"
      --allow-non-kind: "explicit override for non-kind contexts"
      --plan-only: "print ordered summary without contacting a cluster"
    behavior:
      - reads the deterministic inventory from rendered manifests
      - prints MR-comment-friendly object order and target namespace
      - applies objects using kubectl only after guardrail checks
  - name: preview gitops render
    args:
      --dir: "rendered preview directory"
      --out: "GitOps bundle output directory"
    behavior:
      - writes manifests/00-*.yaml through manifests/08-*.yaml
      - writes manifest-inventory.json and kustomization.yaml
      - never records local absolute source paths
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: preview-local-apply-gitops-unit-tests
requirements:
  inventory_order:
    id: R1
    text: "render emits plans/manifest-inventory.json and manifest_inventory_from_dir returns the fixed Kubernetes object order."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test render_contract"
  plan_only:
    id: R2
    text: "preview apply --plan-only prints an ordered MR-friendly summary without contacting a cluster."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test local_cicd_contract"
  gitops_bundle:
    id: R3
    text: "preview gitops render writes a deterministic relative-path bundle with no local absolute paths."
    kind: behavior
    risk: high
    verify: "cargo test -p preview --test local_cicd_contract"
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "manifest inventory order"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "plan-only apply summary"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "GitOps bundle determinism"
      risk: high
      verifymethod: test
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: preview-kind-local-apply
    command: "PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture"
    assertions:
      - "`preview apply` creates the preview namespace and workload objects in kind."
      - "`preview apply --dry-run` performs server-side dry-run after namespace creation."
      - "`preview apply` can re-apply the same rendered directory idempotently."
      - "rollout, endpoint, port-forward, RBAC, quota, and cleanup checks still pass."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "projects/preview/src/apply.rs"
    action: add
    section: logic
    description: "Implement manifest inventory, guardrails, kubectl apply, summary output, and GitOps bundle rendering."
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-preview-src-apply-rs>"
  - path: "projects/preview/src/render.rs"
    action: modify
    section: schema
    description: "Render plans/manifest-inventory.json and expose render_single_manifest for shared inventory generation."
    impl_mode: hand-written
  - path: "projects/preview/src/main.rs"
    action: modify
    section: cli
    description: "Add preview apply and preview gitops render CLI surfaces."
    impl_mode: hand-written
  - path: "projects/preview/src/lib.rs"
    action: modify
    section: schema
    description: "Export apply and GitOps helper types."
    impl_mode: hand-written
  - path: "projects/preview/tests/local_cicd_contract.rs"
    action: modify
    section: unit-test
    description: "Cover ordered inventory, apply plan-only summary, and deterministic GitOps bundle output."
    impl_mode: hand-written
  - path: "projects/preview/tests/render_contract.rs"
    action: modify
    section: unit-test
    description: "Cover rendered manifest inventory presence and ordering."
    impl_mode: hand-written
  - path: "projects/preview/tests/kind_lifecycle.rs"
    action: modify
    section: e2e-test
    description: "Use preview apply for direct apply, server dry-run, and idempotent re-apply in kind."
    impl_mode: hand-written
```
