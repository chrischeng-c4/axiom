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
