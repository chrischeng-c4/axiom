---
id: raft-storage-resize-cli-and-docs
summary: >
  Document that a `spec.serving.raftStorage` CR edit does not, by itself,
  resize existing per-pod PVCs (StatefulSet `volumeClaimTemplates` are
  immutable after creation, for every `replicasPerShard` value post-#812),
  and add a new `lumen k8s operator resize-storage --namespace <ns> --name
  <name> [--dry-run]` CLI helper (behind the existing `operator` feature)
  that lists the instance's live `raft-<name>-<n>` PVCs, compares each to
  the CR's declared size, and patches `spec.resources.requests.storage`
  directly on PVCs whose bound StorageClass allows expansion.
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "lumen-crd-reconcile-loop-kube-rs-operator"
    coverage: partial
    rationale: >
      Issue #809 (re-scoped after #812 landed the unconditional raft PVC)
      closes the remaining capacity-growth gap for this capability's
      long-running-service promise: the operator's rendered
      `volumeClaimTemplates` size is write-once with no documented manual
      procedure and no operator/CLI path to grow it after first apply, for
      any `replicasPerShard` value.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: raft-storage-resize-cli-and-docs
entry: cli_start
nodes:
  cli_start:     { kind: start,    label: "lumen k8s operator resize-storage --namespace <ns> --name <name> [--dry-run] (feature = operator)" }
  fetch_cr:      { kind: process,  label: "Api<Lumen>::namespaced(ns).get(name) -> desired = spec.serving.raftStorage" }
  list_pvcs:     { kind: process,  label: "Api<PersistentVolumeClaim>::namespaced(ns).list(labels app.kubernetes.io/instance=<name>,component=server), keep names starting raft-<name>-" }
  per_pvc:       { kind: decision, label: "for each matching PVC: decide(current, desired)" }
  parse_fail:    { kind: process,  label: "Unparseable -> report only, no mutation" }
  no_op:         { kind: process,  label: "NoOp (current == desired) -> report only, no mutation" }
  shrink:        { kind: process,  label: "ShrinkUnsupported (desired < current) -> report only, no mutation (Kubernetes cannot shrink a bound PVC)" }
  grow:          { kind: decision, label: "Grow (desired > current): resolve pvc.spec.storageClassName, fetch StorageClass" }
  no_expand:     { kind: process,  label: "allowVolumeExpansion != true -> report blocked, no mutation (deployer must recreate the PVC/StatefulSet manually)" }
  can_expand:    { kind: decision, label: "allowVolumeExpansion == true" }
  dry_run_only:  { kind: process,  label: "dry_run == true -> report would-patch, no mutation" }
  patch_pvc:     { kind: process,  label: "Api<PersistentVolumeClaim>.patch(name, Patch::Merge({spec:{resources:{requests:{storage: desired}}}}))" }
  emit_report:   { kind: terminal, label: "print per-PVC outcome table (patched | blocked | no-op | shrink-skipped | unparseable); CR's spec.serving.raftStorage is unchanged by this command (deployer edits the CR separately, same source of truth render() already reads)" }
edges:
  - { from: cli_start,    to: fetch_cr }
  - { from: fetch_cr,     to: list_pvcs }
  - { from: list_pvcs,    to: per_pvc }
  - { from: per_pvc,      to: parse_fail,   label: "Unparseable" }
  - { from: per_pvc,      to: no_op,        label: "NoOp" }
  - { from: per_pvc,      to: shrink,       label: "ShrinkUnsupported" }
  - { from: per_pvc,      to: grow,         label: "Grow" }
  - { from: grow,         to: no_expand,    label: "StorageClass missing / allowVolumeExpansion != true" }
  - { from: grow,         to: can_expand,   label: "StorageClass found" }
  - { from: can_expand,   to: dry_run_only, label: "dry_run" }
  - { from: can_expand,   to: patch_pvc,    label: "!dry_run" }
  - { from: parse_fail,   to: emit_report }
  - { from: no_op,        to: emit_report }
  - { from: shrink,       to: emit_report }
  - { from: no_expand,    to: emit_report }
  - { from: dry_run_only, to: emit_report }
  - { from: patch_pvc,    to: emit_report }
---
flowchart TD
    cli_start([lumen k8s operator resize-storage --namespace ns --name name --dry-run?]) --> fetch_cr[fetch Lumen CR -> desired = spec.serving.raftStorage]
    fetch_cr --> list_pvcs[list live raft-name-N PVCs by label selector]
    list_pvcs --> per_pvc{decide current vs desired}
    per_pvc -->|Unparseable| parse_fail[report only]
    per_pvc -->|NoOp| no_op[report only]
    per_pvc -->|ShrinkUnsupported| shrink[report only: k8s cannot shrink a bound PVC]
    per_pvc -->|Grow| grow{resolve StorageClass.allowVolumeExpansion}
    grow -->|not allowed / missing| no_expand[report blocked: manual PVC/StatefulSet recreation required]
    grow -->|allowed| can_expand{dry_run?}
    can_expand -->|yes| dry_run_only[report would-patch]
    can_expand -->|no| patch_pvc[Patch::Merge spec.resources.requests.storage = desired]
    parse_fail --> emit_report([print per-PVC outcome report])
    no_op --> emit_report
    shrink --> emit_report
    no_expand --> emit_report
    dry_run_only --> emit_report
    patch_pvc --> emit_report
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: raft-storage-resize-cli-and-docs-tests
requirements:
  llm_storage_documents_resize_gap:
    id: R1
    text: "lumen llm storage documents that editing spec.serving.raftStorage on a live CR does not resize existing PVCs (StatefulSet volumeClaimTemplates immutability), the manual kubectl patch pvc procedure, the allowVolumeExpansion: true StorageClass precondition, that shrink is unsupported, and the lumen k8s operator resize-storage helper."
    kind: doc
    risk: low
    verify: test
  parse_and_decide_classify_storage_changes:
    id: R2
    text: "parse_storage_bytes/decide correctly classify grow (20Gi -> 30Gi), no-op (20Gi -> 20Gi), shrink-unsupported (20Gi -> 10Gi), and unparseable quantity strings, with no live-cluster dependency."
    kind: behavior
    risk: medium
    verify: test
  cli_help_documents_resize_storage_flags:
    id: R3
    text: "lumen k8s operator resize-storage --help documents --namespace, --name, and --dry-run, and the verb is reachable only when built with --features operator, consistent with the existing k8s operator run feature gate and fallback error message."
    kind: behavior
    risk: low
    verify: inspection
  default_build_excludes_kube_client:
    id: R4
    text: "cargo build -p lumen with no features still compiles with no kube-rs/k8s-openapi client linked; resize-storage is reachable only via the operator feature, matching run_operator/crd_yaml."
    kind: behavior
    risk: low
    verify: inspection
elements:
  resize_pure_unit_tests:
    kind: test
    path: apps/lumen/src/operator/resize.rs
  spec_cli_unit_tests:
    kind: test
    path: apps/lumen/tests/spec_cli.rs
relations:
  - { from: spec_cli_unit_tests, verifies: llm_storage_documents_resize_gap }
  - { from: resize_pure_unit_tests, verifies: parse_and_decide_classify_storage_changes }
  - { from: resize_pure_unit_tests, verifies: cli_help_documents_resize_storage_flags }
  - { from: resize_pure_unit_tests, verifies: default_build_excludes_kube_client }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "llm storage documents resize gap + procedure"
      risk: low
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "parse_storage_bytes/decide classify grow/no-op/shrink/unparseable"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "resize-storage --help documents flags, feature-gated"
      risk: low
      verifymethod: inspection
    }
    requirement R4 {
      id: R4
      text: "default build excludes kube client"
      risk: low
      verifymethod: inspection
    }
    element resize_pure_unit_tests {
      type: test
    }
    element spec_cli_unit_tests {
      type: test
    }
    spec_cli_unit_tests - satisfies -> R1
    resize_pure_unit_tests - satisfies -> R2
    resize_pure_unit_tests - satisfies -> R3
    resize_pure_unit_tests - satisfies -> R4
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/operator/resize.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "New module: parse_storage_bytes(&str) -> Result<u64> (Kubernetes quantity suffixes Ki/Mi/Gi/Ti and bare bytes), a ResizeAction enum {Grow, NoOp, ShrinkUnsupported, Unparseable} plus decide(current: &str, desired: &str) -> ResizeAction (pure, unit-tested), a PvcResizeOutcome struct (pvc name, action, patched: bool, detail: String), and an impure async resize_instance(client: kube::Client, namespace: &str, name: &str, dry_run: bool) -> Result<Vec<PvcResizeOutcome>> that: fetches the named Lumen CR for spec.serving.raftStorage, lists PVCs labeled app.kubernetes.io/instance=<name> filtered to names starting raft-<name>-, runs decide() per PVC, and for Grow results looks up the bound PersistentVolumeClaim.spec.storage_class_name's StorageClass.spec.allow_volume_expansion — patching spec.resources.requests.storage via Patch::Merge only when allowed and !dry_run, otherwise recording a blocked/would-patch/no-op/shrink-skipped outcome without mutating the PVC."
  - path: apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-resize-rs.md
    action: create
    section: source
    impl_mode: hand-written
    description: "New SPEC-MANAGED rust-source-unit tech-design doc for resize.rs, mirroring the format of the other projects-lumen-src-operator-*-rs.md docs."
  - path: apps/lumen/src/operator/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add `pub mod resize;` alongside the existing `pub mod crd; pub mod lease; pub mod reconcile; pub mod render;` lines."
  - path: apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-mod-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited operator/mod.rs."
  - path: apps/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add K8sOperatorCmd::ResizeStorage(K8sOperatorResizeStorageArgs { namespace: String, name: String, dry_run: bool }) beside Run/Render; wire it in the k8s() dispatcher's K8sCmd::Operator match arm calling a new resize_storage(args) function; add a #[cfg(feature = \"operator\")] real impl (kube::Client::try_default().await, lumen::operator::resize::resize_instance, print a per-PVC outcome table) paired with a #[cfg(not(feature = \"operator\"))] fallback bailing with the same 'rebuild with --features operator' message pattern already used by run_operator/crd_yaml; update the k8s() doc comment to note resize-storage as a second live-cluster verb alongside operator run."
  - path: apps/lumen/tech-design/semantic/source/projects-lumen-src-bin-lumen-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited bin/lumen.rs."
  - path: apps/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Extend llm_storage_md() with a new 'Resizing raftStorage' section: (a) why a plain spec.serving.raftStorage CR edit does not resize existing PVCs (StatefulSet volumeClaimTemplates immutability, applies at every replicasPerShard value post-#812), (b) the manual `kubectl patch pvc raft-<name>-<n> --type merge -p '{\"spec\":{\"resources\":{\"requests\":{\"storage\":\"<new size>\"}}}}'` procedure per pod, requiring allowVolumeExpansion: true on the bound StorageClass, (c) that PVC shrink is unsupported by Kubernetes and requires pod/PVC recreation instead, and (d) the new `lumen k8s operator resize-storage --namespace <ns> --name <name> [--dry-run]` helper as the automated form of the same procedure."
  - path: apps/lumen/tech-design/semantic/source/projects-lumen-src-spec-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited spec.rs."
  - path: apps/lumen/tests/operator_render.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "No render() shape changes needed for this issue, but confirm (via an added lightweight assertion or comment) that the raft PVC volumeClaimTemplate this WI's resize tooling targets is still rendered unconditionally per #812, keeping this fixture file and the new resize tooling in sync."
  - path: apps/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Extend the llm storage doc test to assert the new Resizing raftStorage content (volumeClaimTemplates immutability, kubectl patch pvc procedure, allowVolumeExpansion requirement, shrink unsupported, and the lumen k8s operator resize-storage helper reference)."
  - path: apps/lumen/tech-design/semantic/lumen-tests.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Sync the SPEC-MANAGED Source block byte-for-byte with the edited operator_render.rs and spec_cli.rs."
```
