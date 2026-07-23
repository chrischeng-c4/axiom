---
id: '2467'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-full-width-titlebar-tabs
entry: native-window
nodes:
  native-window: { kind: start, label: "native macOS window" }
  accessory: { kind: process, label: "titlebar accessory host" }
  tab-row: { kind: process, label: "full-width 40pt terminal tabs" }
  terminal: { kind: terminal, label: "terminal content" }
edges:
  - { from: native-window, to: accessory }
  - { from: accessory, to: tab-row }
  - { from: tab-row, to: terminal }
---
flowchart LR
  window([Native window]) --> accessory[Titlebar accessory]
  accessory --> tabs[Full-width terminal tabs]
  tabs --> terminal([Terminal content])
```

An AppKit `NSTitlebarAccessoryViewController` hosts a SwiftUI tab row across the titlebar's content width. It maintains a 40-point row and resizes with the window. The normal terminal workspace no longer owns tab chrome; it begins immediately below the titlebar accessory.

The accessory has no effect on traffic lights, draggable native chrome, project selection, terminal tab identity, PTY lifecycle, or renderer retention.
