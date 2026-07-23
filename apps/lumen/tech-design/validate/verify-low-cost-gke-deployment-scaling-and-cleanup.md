---
id: '2370'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-gke-lumen-only-contract
entry: inputs
nodes:
  inputs: { kind: start, label: Read explicit project, run id, immutable Lumen image, and Lumen-only mode }
  validate: { kind: decision, label: Are project, digest, limits, and empty run scope valid? }
  reject: { kind: terminal, label: Fail before cloud mutation with remediation }
  render: { kind: process, label: Build Lumen CLI and render only Lumen CRD, operator, and instance layers }
  deploy: { kind: process, label: Provision only run-scoped storage/IAM then deploy Lumen }
  evidence: { kind: process, label: Write Lumen acceptance plus cleanup evidence }
  done: { kind: terminal, label: Return passed only after cleanup verification }
edges:
  - { from: inputs, to: validate }
  - { from: validate, to: reject, label: invalid }
  - { from: validate, to: render, label: valid }
  - { from: render, to: deploy }
  - { from: deploy, to: evidence }
  - { from: evidence, to: done }
---
flowchart TD
  inputs([Explicit Lumen-only inputs]) --> validate{Valid digest, limits, and run scope?}
  validate -->|no| reject([Fail before cloud mutation])
  validate -->|yes| render[Render Lumen-only artifacts]
  render --> deploy[Provision and deploy Lumen only]
  deploy --> evidence[Write acceptance and cleanup evidence]
  evidence --> done([Passed only after verify-clean])
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
  - path: benchmarks/gcp-operator-acceptance/scripts/render-manifests.sh
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: rendered service layer selection
  - path: benchmarks/gcp-operator-acceptance/evidence/schema.json
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: terminal acceptance evidence schema
  - path: benchmarks/gcp-operator-acceptance/tests/lumen_only_mode.sh
    action: create
    section: unit-test
    impl_mode: hand-written
    anchor: shell regression for Lumen-only phase selection
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-gke-deployment-scaling-cleanup-verification
requirements:
  capability_evidence:
    id: R4
    text: "A successful current Lumen run is recorded with command, commit, evidence path, and exclusions in the Lumen capability contract."
    kind: regression
    risk: medium
    verify: aw capability check --project lumen --skip-issue-inventory
  lumen_gke_acceptance:
    id: R2
    text: "The live low-cost GKE journey proves Lumen reconcile, restart persistence, GCS upload/readback, disk-driven 1-to-2 split, and cleanup with non-zero work."
    kind: functional
    risk: high
    verify: LUMEN_ONLY=1 benchmarks/gcp-operator-acceptance/scripts/run.sh
  lumen_only_static_contract:
    id: R1
    text: "The reusable harness can select a Lumen-only phase without requiring or deploying Sift."
    kind: regression
    risk: medium
    verify: benchmarks/gcp-operator-acceptance/tests/lumen_only_mode.sh
  offline_harness_gate:
    id: R1
    text: "Shell syntax and Terraform static validation remain reproducible without contacting GCP."
    kind: regression
    risk: medium
    verify: benchmarks/gcp-operator-acceptance/scripts/check.sh
---
flowchart TD
    r1[R1 lumen only static contract] --> benchmarks_gcp_operator_acceptance_tests_lumen_only_mode_sh[benchmarks/gcp-operator-acceptance/tests/lumen_only_mode.sh]
    r1[R1 offline harness gate] --> benchmarks_gcp_operator_acceptance_scripts_check_sh[benchmarks/gcp-operator-acceptance/scripts/check.sh]
    r2[R2 lumen gke acceptance] --> lumen_only_1_benchmarks_gcp_operator_acceptance_scripts_run_sh[LUMEN_ONLY=1 benchmarks/gcp-operator-acceptance/scripts/run.sh]
    r4[R4 capability evidence] --> aw_capability_check_project_lumen_skip_issue_inventory[aw capability check --project lumen --skip-issue-inventory]
```
