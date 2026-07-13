---
id: "1590"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-kind-operator-recovery
entry: build
nodes:
  build: { kind: start, label: "Build Tape image and create Kind" }
  install: { kind: process, label: "Install CRD and operator" }
  reconcile: { kind: process, label: "Apply Tape CR and wait for StatefulSet" }
  append: { kind: process, label: "Append durable event through NodePort" }
  restart: { kind: process, label: "Delete serving pod; retain PVC" }
  verify: { kind: decision, label: "Replay survives and fresh append works?" }
  fail: { kind: terminal, label: "Fail with cluster diagnostics" }
  pass: { kind: terminal, label: "Clean Kind cluster and pass" }
edges:
  - { from: build, to: install }
  - { from: install, to: reconcile }
  - { from: reconcile, to: append }
  - { from: append, to: restart }
  - { from: restart, to: verify }
  - { from: verify, to: fail, label: "no" }
  - { from: verify, to: pass, label: "yes" }
---
flowchart TD
 build[Build image and create Kind] --> install[Install CRD and operator]
 install --> reconcile[Apply Tape CR and wait for StatefulSet]
 reconcile --> append[Append durable event]
 append --> restart[Delete serving pod]
 restart --> verify{Replay and fresh append work?}
 verify -->|no| fail([Emit diagnostics and fail])
 verify -->|yes| pass([Clean cluster and pass])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/scripts/kind-e2e.sh
    action: create
    section: e2e-test
    impl_mode: hand-written
    description: "Build the checked-in Tape image, create a disposable Kind cluster, install the real CRD/operator, exercise append/replay across one pod replacement, and clean up by default. generator gap: missing-generator:service-kind-dogfood (#1590)."
  - path: apps/tape/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Replace the absent-live-Kind caveat with the bounded operator recovery gate and retain explicit non-claims for multi-shard and soak coverage. generator gap: missing-generator:kubernetes-capability-evidence (#1590)."
```
