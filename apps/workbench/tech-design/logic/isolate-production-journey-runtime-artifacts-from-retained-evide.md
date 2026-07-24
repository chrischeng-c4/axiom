---
id: '2502'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-test-artifact-routing
entry: test
nodes:
  test: { kind: start, label: "test run" }
  runtime: { kind: terminal, label: "ignored runtime artifacts" }
  refresh: { kind: process, label: "explicit refresh" }
  retained: { kind: terminal, label: "tracked evidence" }
edges:
  - { from: test, to: runtime }
  - { from: refresh, to: retained }
---
flowchart LR
  test[Test run] --> runtime[Ignored runtime artifacts]
  refresh[Explicit refresh] --> retained[Tracked evidence]
```

Routine tests write run-specific screenshots, transcripts, and measurements below repository-local ignored runtime state. The existing tracked evidence remains the retained review baseline and is updated only through an explicit refresh environment switch.
