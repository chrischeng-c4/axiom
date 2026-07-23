---
id: '2464'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-visible-terminal-tabs
entry: native-window
nodes:
  native-window: { kind: start, label: "native hidden-titlebar window" }
  titlebar: { kind: process, label: "reserve titlebar hit-test region" }
  terminal: { kind: process, label: "place terminal workspace in normal content safe area" }
  tabs: { kind: terminal, label: "visible terminal tab strip" }
edges:
  - { from: native-window, to: titlebar }
  - { from: native-window, to: terminal }
  - { from: titlebar, to: terminal }
  - { from: terminal, to: tabs }
---
flowchart LR
    window([Native window]) --> titlebar[Native titlebar]
    window --> terminal[Terminal content region]
    titlebar --> terminal
    terminal --> tabs([Visible terminal tabs])
```

`terminalWorkspace` stays inside the normal top container safe area. The native titlebar keeps ownership of its hit-test and drawing region, while the tab strip remains the first visible control in the terminal content area.

The recovery does not change project selection, PTY cwd, terminal lifecycle, terminal renderer identity, or terminal-tab identifiers. A future native toolbar integration may change titlebar placement only with a dedicated AppKit integration contract.
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
id: workbench-visible-terminal-tabs-verification
requirements:
  native_package_builds:
    id: R2
    text: "The native macOS package compiles and its test suite passes with the normal terminal content safe area."
    kind: regression
    risk: low
    verify: swift test --package-path apps/workbench/macos
  terminal_tabs_visible:
    id: R1
    text: "The terminal tab strip stays visible below the native titlebar in the Beta application."
    kind: regression
    risk: medium
    verify: WorkbenchMacUITests.testTerminalTabsRemainVisibleBelowTitlebar
---
flowchart TD
    r1[R1 terminal tabs visible] --> workbenchmacuitests_testterminaltabsremainvisiblebelowtitlebar[WorkbenchMacUITests.testTerminalTabsRemainVisibleBelowTitlebar]
    r2[R2 native package builds] --> swift_test_package_path_apps_workbench_macos[swift test --package-path apps/workbench/macos]
```
