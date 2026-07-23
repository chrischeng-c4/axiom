---
id: '2463'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-titlebar-terminal-tabs
entry: native-window
nodes:
  native-window: { kind: start, label: "native hidden-titlebar window" }
  sidebar: { kind: process, label: "preserve sidebar safe area" }
  terminal: { kind: process, label: "extend terminal workspace through top safe area" }
  tabs: { kind: terminal, label: "top-aligned terminal tab strip" }
edges:
  - { from: native-window, to: sidebar }
  - { from: native-window, to: terminal }
  - { from: terminal, to: tabs }
---
flowchart LR
    window([Native window]) --> sidebar[Projects safe area]
    window --> terminal[Terminal top safe-area expansion]
    terminal --> tabs([Top-aligned terminal tabs])
```

Only `terminalWorkspace` extends through the native window top content safe area. `folderSidebar` and `auxiliaryColumn` retain their existing safe-area layouts. The terminal tab strip is therefore the first visual and keyboard-focusable element of the central column.

The layout operation cannot change project selection, PTY cwd, terminal lifecycle, terminal renderer identity, or tab identifiers.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/workbench/macos/Sources/WorkbenchMac/WorkbenchView.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchView
  - path: apps/workbench/macos/UITests/WorkbenchMacUITests.swift
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: WorkbenchMacUITests
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: workbench-titlebar-terminal-tabs-verification
requirements:
  native_layout_builds:
    id: R2
    text: "The native macOS package layout compiles with the titlebar-aligned terminal workspace."
    kind: regression
    risk: low
    verify: swift test --package-path apps/workbench/macos
  terminal_tabs_top_aligned:
    id: R1
    text: "The terminal tab strip begins at the top edge of the central terminal workspace while the sidebar remains below its titlebar control region."
    kind: e2e
    risk: medium
    verify: WorkbenchMacUITests.testTerminalTabsUseTopContentEdge
---
flowchart TD
    r1[R1 terminal tabs top aligned] --> workbenchmacuitests_testterminaltabsusetopcontentedge[WorkbenchMacUITests.testTerminalTabsUseTopContentEdge]
    r2[R2 native layout builds] --> swift_test_package_path_apps_workbench_macos[swift test --package-path apps/workbench/macos]
```
