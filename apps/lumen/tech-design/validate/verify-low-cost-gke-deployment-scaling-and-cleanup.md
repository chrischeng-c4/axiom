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
id: lumen-gke-lumen-only-contract-verification
requirements:
  evidence_contract:
    id: R2
    text: "A Lumen-only terminal record includes mode, current immutable image, Lumen acceptance, exclusions, and mandatory cleanup outcome."
    kind: regression
    risk: medium
    verify: benchmarks/gcp-operator-acceptance/tests/lumen_only_mode.sh
  live_contract:
    id: R2
    text: "The live command proves Lumen work and cleanup under the stated cloud cap without starting the deferred Sift phase."
    kind: functional
    risk: high
    verify: LUMEN_ONLY=1 PROJECT_ID=<project> LUMEN_IMAGE=<immutable-digest> benchmarks/gcp-operator-acceptance/scripts/run.sh
  lumen_only_contract:
    id: R1
    text: "LUMEN_ONLY=1 rejects missing or non-immutable Lumen input and never requires a Sift image, Sift build, or Sift manifest."
    kind: regression
    risk: high
    verify: benchmarks/gcp-operator-acceptance/tests/lumen_only_mode.sh
---
flowchart TD
    r1[R1 lumen only contract] --> benchmarks_gcp_operator_acceptance_tests_lumen_only_mode_sh[benchmarks/gcp-operator-acceptance/tests/lumen_only_mode.sh]
    r2[R2 evidence contract] --> benchmarks_gcp_operator_acceptance_tests_lumen_only_mode_sh
    r2[R2 live contract] --> lumen_only_1_project_id_project_lumen_image_immutable_digest_benchmarks_gcp_operator_acceptance_scripts_run_sh[LUMEN_ONLY=1 PROJECT_ID=<project> LUMEN_IMAGE=<immutable-digest> benchmarks/gcp-operator-acceptance/scripts/run.sh]
```
