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
id: workbench-native-titlebar-terminal-tabs-verification
requirements:
  native_package_builds:
    id: R2
    text: "The native macOS package compiles and its test suite passes with the toolbar-hosted tab strip."
    kind: regression
    risk: low
    verify: swift test --package-path apps/workbench/macos
  native_titlebar_tabs:
    id: R1
    text: "Terminal tabs render in the native toolbar and remain discoverable to accessibility automation."
    kind: regression
    risk: medium
    verify: WorkbenchMacUITests.testTerminalTabsAppearInNativeToolbar
---
flowchart TD
    r1[R1 native titlebar tabs] --> workbenchmacuitests_testterminaltabsappearinnativetoolbar[WorkbenchMacUITests.testTerminalTabsAppearInNativeToolbar]
    r2[R2 native package builds] --> swift_test_package_path_apps_workbench_macos[swift test --package-path apps/workbench/macos]
```
