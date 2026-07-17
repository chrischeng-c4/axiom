---
id: '1884'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: serve-startup-drain-publication
entry: serve_startup
nodes:
  receivers: { kind: process, label: "construct admin and both plane drain receivers" }
  signal_task: { kind: process, label: "spawn SIGTERM/SIGINT watcher after receiver construction" }
  publish: { kind: process, label: "send_replace Draining even with no receiver" }
  wait: { kind: decision, label: "shutdown observes current Draining or waits for transition" }
  stopped: { kind: terminal, label: "both planes stop before serving new work" }
edges:
  - { from: serve_startup, to: receivers }
  - { from: receivers, to: signal_task }
  - { from: signal_task, to: publish, label: signal or drain route }
  - { from: publish, to: wait }
  - { from: wait, to: stopped }
---
flowchart TD
  start["serve startup"] --> receivers["construct admin + TCP + admin receivers"]
  receivers --> signal["spawn signal watcher"]
  signal --> publish["send_replace Draining"]
  publish --> wait{"shutdown sees current drain?"}
  wait -->|yes or changed| stop["both planes stop"]
```
