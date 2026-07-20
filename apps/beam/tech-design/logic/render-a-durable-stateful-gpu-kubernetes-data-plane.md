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
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: 2154-verification
requirements:
  render_backup_cronjob:
    id: R3
    text: "Render a scheduled object-storage backup job/policy for production profiles."
    kind: functional
    risk: high
    verify: cargo test -p beam --test k8s_render
  render_gpu_and_extras:
    id: R2
    text: "Render GPU resource requests/node placement plus standard probes, Service, PDB, security context, and auth-secret wiring."
    kind: functional
    risk: high
    verify: cargo test -p beam --test k8s_render
  render_statefulset_pvc:
    id: R1
    text: "Define Beam's service-k8s data-plane renderer as a StatefulSet with stable identity and durable PVC-backed data directory."
    kind: functional
    risk: high
    verify: cargo test -p beam --test k8s_render
---
flowchart TD
    r1[R1 render statefulset pvc] --> cargo_test_p_beam_test_k8s_render[cargo test -p beam --test k8s_render]
    r2[R2 render gpu and extras] --> cargo_test_p_beam_test_k8s_render
    r3[R3 render backup cronjob] --> cargo_test_p_beam_test_k8s_render
```
