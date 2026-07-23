---
id: '2466'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-native-titlebar-terminal-tabs
entry: app-window
nodes:
  app-window: { kind: start, label: "hidden-titlebar native window" }
  toolbar: { kind: process, label: "native unified compact toolbar" }
  tab-strip: { kind: process, label: "principal toolbar terminal tabs" }
  content: { kind: terminal, label: "terminal content below toolbar" }
edges:
  - { from: app-window, to: toolbar }
  - { from: toolbar, to: tab-strip }
  - { from: tab-strip, to: content }
---
flowchart LR
  window([Native window]) --> toolbar[Native toolbar]
  toolbar --> tabs[Terminal tabs]
  tabs --> content([Terminal content])
```

The terminal tab strip is a SwiftUI `ToolbarItem` in the native `.principal` toolbar placement. AppKit owns traffic lights and unused titlebar space; the tab strip owns only normal interactive controls. The terminal body begins below the toolbar and no content view ignores its safe area.

This layout cannot change project selection, terminal tab identity, PTY lifecycle, terminal renderer retention, or auxiliary-column ordering.
