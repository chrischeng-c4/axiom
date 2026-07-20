---
id: '2154'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: render-a-durable-stateful-gpu-kubernetes-data-plane
entry: start
nodes:
  start: { kind: start, label: "Render Data Plane" }
  render_statefulset: { kind: process, label: "Render StatefulSet with VolumeClaimTemplates" }
  mount_pvc: { kind: process, label: "Mount Data Directory PVC" }
  mount_auth: { kind: process, label: "Mount Auth Secret and Env" }
  add_gpu: { kind: process, label: "Add GPU Resource Requests and Placement Constraints" }
  add_probes_pdb: { kind: process, label: "Add Probes, Security Context, and PDB" }
  add_backup: { kind: process, label: "Render Backup CronJob (Prod)" }
  done: { kind: terminal, label: "Finished" }
edges:
  - { from: start, to: render_statefulset }
  - { from: render_statefulset, to: mount_pvc }
  - { from: mount_pvc, to: mount_auth }
  - { from: mount_auth, to: add_gpu }
  - { from: add_gpu, to: add_probes_pdb }
  - { from: add_probes_pdb, to: add_backup }
  - { from: add_backup, to: done }
---
flowchart TD
    start([Render Data Plane]) --> render_statefulset[Render StatefulSet with VolumeClaimTemplates]
    render_statefulset --> mount_pvc[Mount Data Directory PVC]
    mount_pvc --> mount_auth[Mount Auth Secret and Env]
    mount_auth --> add_gpu[Add GPU Resource Requests and Placement Constraints]
    add_gpu --> add_probes_pdb[Add Probes, Security Context, and PDB]
    add_probes_pdb --> add_backup[Render Backup CronJob Prod]
    add_backup --> done([Finished])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/src/operator/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "pub struct BeamSpec"
  - path: apps/beam/src/operator/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "impl ManagedService for Beam"
  - path: apps/beam/src/dx.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: render_instance_yaml
  - path: apps/beam/tests/k8s_render.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: "tests"
```
