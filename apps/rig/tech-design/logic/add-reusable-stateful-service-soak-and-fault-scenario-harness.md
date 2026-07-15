---
id: '1645'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: rig-stateful-service-harness
entry: warmup
nodes:
  warmup: { kind: start, label: "Run bounded warm-up action" }
  observe: { kind: process, label: "Capture steady-state observations and evidence" }
  fault: { kind: process, label: "Inject one bounded service fault or restart" }
  recover: { kind: process, label: "Recover or restart the service" }
  verify: { kind: process, label: "Assert domain-specific state continuity" }
  failed: { kind: process, label: "Record failed phase, error, duration, and retained evidence" }
  teardown: { kind: process, label: "Always run bounded teardown and capture its outcome" }
  report: { kind: terminal, label: "Emit deterministic typed scenario report" }
edges:
  - { from: warmup, to: observe, label: "pass" }
  - { from: observe, to: fault, label: "pass" }
  - { from: fault, to: recover, label: "pass" }
  - { from: recover, to: verify, label: "pass" }
  - { from: verify, to: teardown, label: "pass" }
  - { from: warmup, to: failed, label: "fail or timeout" }
  - { from: observe, to: failed, label: "fail or timeout" }
  - { from: fault, to: failed, label: "fail or timeout" }
  - { from: recover, to: failed, label: "fail or timeout" }
  - { from: verify, to: failed, label: "fail or timeout" }
  - { from: failed, to: teardown }
  - { from: teardown, to: report }
---
flowchart TD
    warmup([Run bounded warm-up action]) -->|pass| observe[Capture steady-state observations and evidence]
    observe -->|pass| fault[Inject one bounded service fault or restart]
    fault -->|pass| recover[Recover or restart the service]
    recover -->|pass| verify[Assert domain-specific state continuity]
    warmup -->|fail or timeout| failed[Record failed phase error duration and retained evidence]
    observe -->|fail or timeout| failed
    fault -->|fail or timeout| failed
    recover -->|fail or timeout| failed
    verify -->|fail or timeout| failed
    verify -->|pass| teardown[Always run bounded teardown and capture its outcome]
    failed --> teardown
    teardown --> report([Emit deterministic typed scenario report])
```
