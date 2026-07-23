---
id: '2470'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-app-owned-terminal-chrome
entry: native-window
nodes:
  native-window: { kind: start, label: "native macOS window" }
  content: { kind: process, label: "full-size app-owned content" }
  columns: { kind: process, label: "aligned Projects terminal Auxiliary columns" }
  tabs: { kind: terminal, label: "always-visible terminal tabs" }
edges:
  - { from: native-window, to: content }
  - { from: content, to: columns }
  - { from: columns, to: tabs }
---
flowchart LR
  window([Native window]) --> content[App-owned top chrome]
  content --> columns[Three aligned columns]
  columns --> tabs([Always-visible terminal tabs])
```

The native window uses a transparent titlebar and full-size content view, while the terminal tab strip stays inside the SwiftUI terminal workspace. The root workspace therefore reaches the same top edge in the Projects, terminal, and Auxiliary columns. In both normal and macOS fullscreen modes, the tab strip is app content and never depends on native toolbar or titlebar hover reveal.

The Project column retains a narrow leading titlebar-safe region for traffic-light controls in a normal window. The center column owns a compact 40-point tab header, with ordinary whole-tab selection, independent close controls, keyboard shortcuts, and the add-shell action.

The layout operation cannot change project selection, PTY cwd, terminal lifecycle, renderer retention, tab identifiers, or Auxiliary file-listing state.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/workbench/macos/Sources/WorkbenchMac/WorkbenchMacApp.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchMacApp
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
id: workbench-app-owned-terminal-chrome-verification
requirements:
  beta_snapshot:
    id: R3
    text: "The beta runtime exposes a local content snapshot with the visible terminal tab strip."
    kind: e2e
    risk: medium
    verify: workbench snapshot --out /private/tmp/workbench-beta.png
  fullscreen_visible_tabs:
    id: R1
    text: "Terminal tabs remain in the app content region and are visible without native titlebar reveal."
    kind: regression
    risk: medium
    verify: WorkbenchMacUITests.testTerminalTabsRemainInContentChrome
  package_builds:
    id: R2
    text: "The native macOS package compiles and its test suite passes with the app-owned terminal chrome."
    kind: regression
    risk: low
    verify: swift test --package-path apps/workbench/macos
---
flowchart TD
    r1[R1 fullscreen visible tabs] --> workbenchmacuitests_testterminaltabsremainincontentchrome[WorkbenchMacUITests.testTerminalTabsRemainInContentChrome]
    r2[R2 package builds] --> swift_test_package_path_apps_workbench_macos[swift test --package-path apps/workbench/macos]
    r3[R3 beta snapshot] --> workbench_snapshot_out_private_tmp_workbench_beta_png[workbench snapshot --out /private/tmp/workbench-beta.png]
```
