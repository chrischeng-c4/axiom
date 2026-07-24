---
id: '2500'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-recursive-pane-renderer-applicability
entry: pane-tree
nodes:
  tree: { kind: start, label: "project pane tree" }
  leaf: { kind: process, label: "stable terminal leaf" }
  split: { kind: process, label: "recursive split container" }
  control: { kind: process, label: "explicit add or split menu" }
  surface: { kind: terminal, label: "existing SwiftTerm host" }
edges:
  - { from: tree, to: leaf }
  - { from: tree, to: split }
  - { from: split, to: leaf }
  - { from: control, to: split }
  - { from: leaf, to: surface }
---
flowchart LR
  tree([Project pane tree]) --> leaf[Stable terminal leaf]
  tree --> split[Recursive split container]
  split --> leaf
  control[Explicit add / split menu] --> split
  leaf --> surface([Existing SwiftTerm host])
```

This design applies only to the native macOS presentation of the project-scoped `TerminalPaneTree` introduced by #2499. It replaces the flat pane `HStack` with a recursive SwiftUI renderer, exposes explicit Split Right and Split Down profile actions, and allows a split ratio to be updated without changing terminal-session identity. Rust remains the sole PTY owner and the existing `TerminalSurface` continues to render bytes for the referenced `TerminalTab`.

Drag-to-split, worktree lifecycle, persisted restart restoration, tabs inside a pane, and Auxiliary feature expansion remain outside this slice.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/workbench/macos/Sources/WorkbenchMacCore/WorkbenchModel.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchModel
  - path: apps/workbench/macos/Sources/WorkbenchMac/WorkbenchView.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchView
  - path: apps/workbench/macos/Tests/WorkbenchMacCoreTests/WorkbenchModelTests.swift
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: WorkbenchModelTests
  - path: apps/workbench/macos/UITests/WorkbenchMacUITests.swift
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: WorkbenchMacUITests
```
