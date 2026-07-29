---
id: '2370'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-gke-acceptance-mode-contract
entry: inputs
nodes:
  inputs: { kind: start, label: Read explicit project, run id, immutable images, and ACCEPTANCE_APPS mode }
  validate: { kind: decision, label: Are project, digest, limits, and the mode enum valid? }
  reject: { kind: terminal, label: Fail before cloud mutation with remediation }
  render: { kind: process, label: Build the mode's CLI and render only that mode's CRD, operator, and instance layers }
  deploy: { kind: process, label: Provision only run-scoped storage/IAM then deploy the mode's apps }
  evidence: { kind: process, label: Write per-app acceptance plus cleanup evidence }
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
  inputs([Explicit mode-scoped inputs]) --> validate{Valid digest, limits, and mode enum?}
  validate -->|no| reject([Fail before cloud mutation])
  validate -->|yes| render[Render only the selected mode's artifacts]
  render --> deploy[Provision and deploy the selected mode]
  deploy --> evidence[Write acceptance and cleanup evidence]
  evidence --> done([Passed only after verify-clean])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: acceptance/gcp/scripts/run.sh
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: top-level run harness configuration and phase dispatch
  - path: acceptance/gcp/scripts/render-manifests.sh
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: rendered service layer selection
  - path: acceptance/gcp/evidence/schema.json
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: terminal acceptance evidence schema
  - path: acceptance/gcp/tests/acceptance_mode_selection.sh
    action: create
    section: unit-test
    impl_mode: hand-written
    anchor: shell regression for acceptance-mode phase selection
  - path: acceptance/gcp/scripts/check.sh
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: static acceptance gate entrypoint
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-gke-acceptance-mode-contract-verification
requirements:
  mode_selection_contract:
    id: R1
    text: "ACCEPTANCE_APPS is a closed enum -- 'lumen sift' (default) or 'tape' -- rejected before any cloud mutation, and run.sh and render-manifests.sh close it identically so the harness can never render one app's manifests and verify another's."
    kind: regression
    risk: high
    verify: acceptance/gcp/scripts/check.sh
  cleanup_contract:
    id: R2
    text: "Cleanup is armed on every exit path: the EXIT trap plus the run_completed sentinel, and every namespace the run can create -- including the fleet leg's data-plane namespaces -- is named by the sweep, so a leg that fails midway cannot leak a billable Persistent Disk on the persistent cluster."
    kind: regression
    risk: high
    verify: acceptance/gcp/scripts/check.sh
  evidence_contract:
    id: R2
    text: "The terminal acceptance record is schema-valid and carries run id, project, region, zone, run-scoped backup bucket, and the per-app acceptance block for the selected mode."
    kind: regression
    risk: medium
    verify: acceptance/gcp/scripts/check.sh
  live_contract:
    id: R2
    text: "The live command proves the selected mode's deployment, scaling, and cleanup under the stated cloud cap, using immutable image digests supplied by the caller."
    kind: functional
    risk: high
    verify: PROJECT_ID=<project> LUMEN_IMAGE=<immutable-digest> SIFT_IMAGE=<immutable-digest> bash acceptance/gcp/scripts/run.sh
---
flowchart TD
    r1[R1 mode selection contract] --> acceptance_gcp_scripts_check_sh[acceptance/gcp/scripts/check.sh]
    r2[R2 cleanup contract] --> acceptance_gcp_scripts_check_sh
    r2[R2 evidence contract] --> acceptance_gcp_scripts_check_sh
    r2[R2 live contract] --> project_id_project_lumen_image_immutable_digest_sift_image_immutable_digest_bash_acceptance_gcp_scripts_run_sh[PROJECT_ID=<project> LUMEN_IMAGE=<immutable-digest> SIFT_IMAGE=<immutable-digest> bash acceptance/gcp/scripts/run.sh]
```
