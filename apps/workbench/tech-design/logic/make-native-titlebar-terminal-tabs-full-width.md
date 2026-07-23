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
id: workbench-full-width-titlebar-tabs-verification
requirements:
  full_width_titlebar_tabs:
    id: R1
    text: "Terminal tabs are attached to a native titlebar accessory and remain visible above terminal content."
    kind: regression
    risk: medium
    verify: WorkbenchMacUITests.testTerminalTabsUseFullWidthTitlebarAccessory
  native_package_builds:
    id: R2
    text: "The native macOS package compiles and its test suite passes with the full-width titlebar tab host."
    kind: regression
    risk: low
    verify: swift test --package-path apps/workbench/macos
---
flowchart TD
    r1[R1 full width titlebar tabs] --> workbenchmacuitests_testterminaltabsusefullwidthtitlebaraccessory[WorkbenchMacUITests.testTerminalTabsUseFullWidthTitlebarAccessory]
    r2[R2 native package builds] --> swift_test_package_path_apps_workbench_macos[swift test --package-path apps/workbench/macos]
```
