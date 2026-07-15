---
id: '1485'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: convergence-self-heal
entry: complete
nodes:
  complete: { kind: start, label: "Complete-phase shard-map convergence tick" }
  rollout: { kind: decision, label: "StatefulSet rollout converged?" }
  pod_version: { kind: decision, label: "every serving pod reports target map version?" }
  persist_wait: { kind: process, label: "persist convergenceWaitStartedAt when absent; keep write fence armed" }
  stalled: { kind: decision, label: "persisted stall budget exceeded?" }
  restarted: { kind: decision, label: "convergenceRemediationRestartCount == 0?" }
  claim_restart: { kind: process, label: "write-ahead patch restart count to 1, then trigger one rolling restart" }
  keep_stalled: { kind: process, label: "retain stalled condition, persisted wait, count=1, and fence" }
  wait: { kind: terminal, label: "AwaitingTopologyConvergence" }
  clear: { kind: process, label: "clear wait timestamp/count/stalled condition and write fence" }
  done: { kind: terminal, label: "TopologyConverged" }
edges:
  - { from: complete, to: rollout }
  - { from: rollout, to: persist_wait, label: "no" }
  - { from: rollout, to: pod_version, label: "yes" }
  - { from: pod_version, to: clear, label: "yes" }
  - { from: pod_version, to: persist_wait, label: "no: version mismatch" }
  - { from: persist_wait, to: stalled }
  - { from: stalled, to: wait, label: "no" }
  - { from: stalled, to: restarted, label: "yes" }
  - { from: restarted, to: claim_restart, label: "yes" }
  - { from: restarted, to: keep_stalled, label: "no" }
  - { from: claim_restart, to: wait }
  - { from: keep_stalled, to: wait }
  - { from: clear, to: done }
---
flowchart TD
    complete([Complete-phase convergence tick]) --> rollout{StatefulSet rollout converged?}
    rollout -->|no| persist_wait[persist wait start; keep fence armed]
    rollout -->|yes| pod_version{all serving pods report target map version?}
    pod_version -->|yes| clear[clear persisted wait, restart count, stalled condition, and fence]
    pod_version -->|no| persist_wait
    persist_wait --> stalled{persisted stall budget exceeded?}
    stalled -->|no| wait([AwaitingTopologyConvergence])
    stalled -->|yes| restarted{remediation restart count is zero?}
    restarted -->|yes| claim_restart[write-ahead count=1, then trigger one rolling restart]
    restarted -->|no| keep_stalled[retain stalled condition and fence]
    claim_restart --> wait
    keep_stalled --> wait
    clear --> done([TopologyConverged])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/operator/crd.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Persist convergenceWaitStartedAt and convergenceRemediationRestartCount in the CR workflow/status projection so the stall budget and remediation claim survive operator restarts."
  - path: apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-crd-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep the CRD source mirror aligned with the persisted convergence fields."
  - path: apps/lumen/src/operator/reshard_driver.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the persisted wait start, write-ahead claim one bounded remediation restart on a rollout-complete version mismatch, and retain the fence/stalled condition until convergence."
  - path: apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-reshard-driver-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Keep the reshard-driver source mirror aligned with bounded self-heal behavior."
  - path: apps/lumen/tests/reshard_driver_e2e.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Verify exactly-one remediation, write-ahead durability, operator-restart persistence, fence retention, and eventual convergence."
```
