---
id: '2370'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-gke-deployment-scaling-cleanup
entry: preflight
nodes:
  preflight: { kind: start, label: Run offline harness and local Lumen gates }
  lumen_only: { kind: process, label: Select Lumen-only immutable-image mode; do not deploy Sift }
  provision: { kind: process, label: Reuse Standard GKE and create run-scoped GCS plus Workload Identity }
  operator: { kind: process, label: Render with Lumen CLI then prove reconcile, drift repair, and lease handoff }
  persistence: { kind: process, label: Index non-empty document, restart PVC-backed pod, then query it }
  backup: { kind: process, label: Run backup CronJob; require non-empty GCS object and readback JSON }
  split: { kind: process, label: Set test-only one-byte disk trigger; require 1 to 2 converged shards and query }
  cleanup: { kind: process, label: Run unconditional cleanup and verify no run-scoped cloud or Kubernetes resources }
  pass: { kind: terminal, label: Record exact current evidence in Lumen capability contract }
  classify: { kind: decision, label: Does any gate fail? }
  shared_fix: { kind: process, label: Repair shared owner or harness then rerun from preflight }
  domain_issue: { kind: process, label: Record one bounded Lumen-domain issue and tracked skip }
edges:
  - { from: preflight, to: lumen_only }
  - { from: lumen_only, to: provision }
  - { from: provision, to: operator }
  - { from: operator, to: persistence }
  - { from: persistence, to: backup }
  - { from: backup, to: split }
  - { from: split, to: cleanup }
  - { from: cleanup, to: classify }
  - { from: classify, to: pass, label: no }
  - { from: classify, to: shared_fix, label: shared or non-domain }
  - { from: shared_fix, to: preflight }
  - { from: classify, to: domain_issue, label: Lumen domain }
---
flowchart TD
  preflight([Offline and local gates]) --> lumen_only[Lumen-only immutable image mode]
  lumen_only --> provision[Run-scoped GCS and Workload Identity]
  provision --> operator[Operator reconcile, drift repair, leader handoff]
  operator --> persistence[Index, PVC-backed restart, query]
  persistence --> backup[Backup CronJob, GCS object and readback]
  backup --> split[Test-only disk trigger, 1 to 2 shard convergence]
  split --> cleanup[Unconditional cleanup and verify-clean]
  cleanup --> classify{Any failure?}
  classify -->|no| pass([Record current evidence in CAPABILITIES])
  classify -->|shared/non-domain| shared_fix[Repair canonical shared owner]
  shared_fix --> preflight
  classify -->|Lumen domain| domain_issue[Bounded issue and tracked skip]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: benchmarks/gcp-operator-acceptance/scripts/run.sh
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: top-level run harness configuration and phase dispatch
  - path: benchmarks/gcp-operator-acceptance/scripts/check.sh
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: offline harness validation sequence
  - path: benchmarks/gcp-operator-acceptance/tests/lumen_only_mode.sh
    action: create
    section: unit-test
    impl_mode: hand-written
    anchor: shell regression for Lumen-only phase selection
  - path: benchmarks/gcp-operator-acceptance/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: acceptance boundary and exact lifecycle
  - path: apps/lumen/CAPABILITIES.md
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: cross-service delivery verification work-root evidence
```
