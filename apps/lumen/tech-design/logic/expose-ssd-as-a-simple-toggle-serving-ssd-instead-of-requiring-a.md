---
id: raft-storage-class-ssd-guidance-docs
summary: >
  Documentation-only fix: strengthen the `ServingSpec.raft_storage_class`
  doc comment in `crd.rs` and extend the `lumen llm storage` topic with
  explicit guidance that an unset/cluster-default StorageClass is commonly
  not SSD-backed, and that raft/WAL write latency benefits from picking an
  SSD-backed StorageClass explicitly via the existing `raftStorageClass`
  field. No new CRD field, no render.rs change, no provider-detection
  logic — a per-cloud-provider StorageClass mapping was explicitly rejected
  as a maintenance/correctness liability; a short list of well-known
  example StorageClass names is included as informational reference text
  only.
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "lumen-crd-reconcile-loop-kube-rs-operator"
    coverage: partial
    rationale: >
      Issue #810 closes a deployer-awareness gap for this capability's
      long-running-service promise: the raft/WAL PVC's StorageClass choice
      materially affects write latency, but neither the CRD field doc nor
      `lumen llm storage` said so, leaving a deployer with no signal to
      pick an SSD-backed class before shipping.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: raft-storage-class-ssd-guidance-docs
entry: read_field_doc
nodes:
  read_field_doc: { kind: start,    label: "Deployer reads ServingSpec.raftStorageClass doc comment (crd.rs) or lumen llm storage" }
  check_set:      { kind: decision, label: "spec.serving.raftStorageClass set on the CR?" }
  cluster_default:{ kind: process,  label: "Unset -> render() omits storageClassName on the raft PVC -> Kubernetes binds the cluster default StorageClass, commonly NOT SSD-backed (e.g. GKE standard-rwo/pd-balanced)" }
  latency_check:  { kind: decision, label: "Workload cares about raft/WAL write latency?" }
  accept_default: { kind: terminal, label: "Deployer accepts the cluster default (no action) - acceptable when write latency is not a concern" }
  consult_examples:{ kind: process, label: "Consult lumen llm storage example StorageClass names per provider (GKE premium-rwo/pd-ssd, EKS gp3, AKS managed-csi-premium) as reference only, then verify against the target cluster's actual classes" }
  set_field:      { kind: process,  label: "Deployer sets spec.serving.raftStorageClass to the verified SSD-backed StorageClass name" }
  render_pin:      { kind: process, label: "render()'s existing (unchanged) behavior: raft volumeClaimTemplate.storageClassName is pinned to the declared value when set" }
  ssd_pinned:     { kind: terminal, label: "raft PVC bound to an explicit SSD-backed StorageClass" }
edges:
  - { from: read_field_doc,   to: check_set }
  - { from: check_set,        to: cluster_default,  label: "unset" }
  - { from: check_set,        to: render_pin,        label: "set" }
  - { from: cluster_default,  to: latency_check }
  - { from: latency_check,    to: accept_default,   label: "no" }
  - { from: latency_check,    to: consult_examples, label: "yes" }
  - { from: consult_examples, to: set_field }
  - { from: set_field,        to: render_pin }
  - { from: render_pin,       to: ssd_pinned }
---
flowchart TD
    read_field_doc([Deployer reads raftStorageClass doc comment or lumen llm storage]) --> check_set{spec.serving.raftStorageClass set?}
    check_set -->|unset| cluster_default[render omits storageClassName -> cluster default binds, commonly NOT SSD e.g. GKE standard-rwo]
    check_set -->|set| render_pin[render pins raft PVC storageClassName to declared value - unchanged behavior]
    cluster_default --> latency_check{raft/WAL write latency a concern?}
    latency_check -->|no| accept_default([Accept cluster default - no action])
    latency_check -->|yes| consult_examples[Consult lumen llm storage example StorageClass names per provider - reference only, verify against real cluster]
    consult_examples --> set_field[Set spec.serving.raftStorageClass to verified SSD-backed name]
    set_field --> render_pin
    render_pin --> ssd_pinned([raft PVC bound to explicit SSD-backed StorageClass])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: raft-storage-class-ssd-guidance-docs-tests
requirements:
  crd_doc_comment_warns_cluster_default_not_ssd:
    id: R1
    text: "ServingSpec.raft_storage_class's doc comment states that unset means cluster default, that the cluster default is commonly not SSD-backed (e.g. GKE standard-rwo), and recommends an explicit SSD-backed StorageClass for raft/WAL write latency."
    kind: doc
    risk: low
    verify: inspection
  llm_storage_documents_ssd_guidance:
    id: R2
    text: "lumen llm storage documents the same cluster-default-is-usually-not-SSD guidance plus example StorageClass names for GKE, EKS, and AKS, framed as reference only."
    kind: doc
    risk: low
    verify: test
  no_schema_or_render_change:
    id: R3
    text: "ServingSpec's field set/types/defaults and render.rs are unchanged by this WI - no new CRD field is added."
    kind: behavior
    risk: low
    verify: inspection
elements:
  crd_rs_doc_comment:
    kind: doc
    path: apps/lumen/src/operator/crd.rs
  spec_cli_unit_tests:
    kind: test
    path: apps/lumen/tests/spec_cli.rs
relations:
  - { from: crd_rs_doc_comment,    verifies: crd_doc_comment_warns_cluster_default_not_ssd }
  - { from: spec_cli_unit_tests,   verifies: llm_storage_documents_ssd_guidance }
  - { from: crd_rs_doc_comment,    verifies: no_schema_or_render_change }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "raftStorageClass doc comment warns cluster default is usually not SSD"
      risk: low
      verifymethod: inspection
    }
    requirement R2 {
      id: R2
      text: "lumen llm storage documents SSD guidance + provider examples"
      risk: low
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "no CRD schema field added, render.rs unchanged"
      risk: low
      verifymethod: inspection
    }
    element crd_rs_doc_comment {
      type: doc
    }
    element spec_cli_unit_tests {
      type: test
    }
    crd_rs_doc_comment - satisfies -> R1
    spec_cli_unit_tests - satisfies -> R2
    crd_rs_doc_comment - satisfies -> R3
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/operator/crd.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Strengthen the ServingSpec.raft_storage_class doc comment: state that unset means cluster default, that the cluster default is commonly not SSD-backed (e.g. GKE's standard-rwo), and that raft/WAL write latency benefits from picking an SSD-backed StorageClass explicitly via this field. No schema/type/default change - the field stays Option<String>."
  - path: apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited crd.rs doc comment."
  - path: apps/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Extend llm_storage_md() with a new subsection (same style/placement pattern the #808 backup and #809 resize subsections used) stating that the cluster-default StorageClass is usually not SSD-backed, that raft/WAL write latency benefits from setting spec.serving.raftStorageClass explicitly, and listing a few well-known example StorageClass names per common provider (GKE premium-rwo/pd-ssd, EKS gp3, AKS managed-csi-premium) as informational reference text a deployer should verify against their own cluster - not a mapping the operator consumes or validates."
  - path: apps/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited spec.rs."
  - path: apps/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add a new llm_storage_documents_* test (matching the existing naming/assertion pattern) asserting the new SSD guidance text and per-provider example StorageClass names are present in llm_storage_md()."
  - path: apps/lumen/tech-design/semantic/lumen-tests.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited spec_cli.rs."
```
