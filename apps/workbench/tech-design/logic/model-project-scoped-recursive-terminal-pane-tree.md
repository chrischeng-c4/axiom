---
id: '2499'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-recursive-pane-model
entry: project-workspace
nodes:
  workspace: { kind: start, label: "project workspace" }
  leaf: { kind: process, label: "empty or session leaf" }
  split: { kind: process, label: "horizontal or vertical split" }
  sidecar: { kind: terminal, label: "existing PTY session" }
edges:
  - { from: workspace, to: leaf }
  - { from: leaf, to: split, label: "explicit split only" }
  - { from: leaf, to: sidecar, label: "one session" }
---
flowchart LR
  workspace[Project workspace] --> leaf[Empty or session leaf]
  leaf -->|explicit split only| split[Horizontal / vertical split]
  leaf -->|one session| sidecar([Existing PTY])
```

`WorkbenchModel` owns one in-memory recursive layout tree per project. A leaf references zero or one existing `TerminalTab`; that tab id remains the sole key routed to the Rust sidecar. A split owns orientation and a bounded first-child ratio.

Opening a profile into a nonempty leaf is rejected unless an explicit horizontal or vertical split placement is supplied. Project and pane selection are presentation transitions only: they never launch, shut down, resize, input, or poll a sidecar. Moving or closing the last session in a branch collapses it, leaving one empty root leaf when a workspace has no sessions. Sidecar ids remain ASCII-safe and stable for each session lifetime.

This slice does not render the tree, persist it across relaunch, create pane-internal tab groups, or introduce drag recognition.
