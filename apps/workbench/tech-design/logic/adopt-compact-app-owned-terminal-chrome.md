---
id: '2470'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-app-owned-terminal-chrome-contract
entry: window-attach
nodes:
  window-attach: { kind: start, label: "SwiftUI window attached" }
  configure: { kind: process, label: "configure transparent full-size titlebar" }
  render: { kind: process, label: "render app content through top edge" }
  visible: { kind: terminal, label: "tabs visible fullscreen" }
edges:
  - { from: window-attach, to: configure }
  - { from: configure, to: render }
  - { from: render, to: visible }
---
flowchart LR
  attach([Window attached]) --> configure[Transparent full-size chrome]
  configure --> render[App-owned three-column content]
  render --> visible([Fullscreen-visible tabs])
```

`WorkbenchMacApp` installs an AppKit bridge that configures only the hosting `NSWindow`: `titlebarAppearsTransparent` is enabled, the title visibility is hidden, and `fullSizeContentView` is present. The bridge is idempotent, runs on the main actor, and changes no window state after the first successful attachment.

`WorkbenchView` draws its `NavigationSplitView` through the top container safe area. The Project, terminal, and optional Beta Auxiliary columns therefore begin at the same y-origin. The terminal tab strip remains the first child of `terminalWorkspace`; it owns the terminal action controls and stays visible when macOS hides native chrome in fullscreen.

The Project column includes a titlebar-leading clearance in windowed mode only for traffic-light safety. Its project buttons, terminal tab state, PTY sessions, files listing, and runtime profile remain unchanged.
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
