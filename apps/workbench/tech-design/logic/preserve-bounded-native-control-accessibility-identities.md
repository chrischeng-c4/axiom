---
id: '2511'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-native-accessibility-identities
entry: container
nodes:
  container: { kind: start, label: "SwiftUI container" }
  contain: { kind: process, label: "contain accessibility children" }
  identity: { kind: process, label: "container identifier" }
  child: { kind: terminal, label: "independent descendant identifier" }
  file: { kind: process, label: "bounded top-level file name" }
edges:
  - { from: container, to: contain }
  - { from: contain, to: identity }
  - { from: contain, to: child }
  - { from: file, to: child, label: "auxiliary.file.<name>" }
---
flowchart LR
  container([SwiftUI container]) --> contain[Contain accessibility children]
  contain --> identity[Container identifier]
  contain --> child([Independent descendant identifier])
  file[Bounded top-level file name] -->|auxiliary.file.name| child
```

Apply `.accessibilityElement(children: .contain)` before assigning identifiers to the terminal workspace, toolbar, pane tree, split containers, and pane leaves. This prevents parent identifiers from replacing descendant Menu/Button identifiers in the native accessibility tree. Auxiliary rows identify `auxiliary.file.<entry.name>` because top-level filesystem names are unique within the displayed root and do not inherit an arbitrarily long XCTest container path.
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
id: workbench-native-accessibility-identities-verification
requirements:
  bounded_file_rows:
    id: R2
    text: "Auxiliary file rows use bounded name-based identifiers that remain below XCUI query limits."
    kind: regression
    risk: medium
    verify: WorkbenchMacUITests.testFilesAuxiliaryColumnShowsFixtureEntries
  independent_terminal_controls:
    id: R1
    text: "Terminal workspace, toolbar, and add-profile control retain independent queryable identifiers."
    kind: regression
    risk: high
    verify: WorkbenchMacUITests.testPaneToolbarRemainsInContentChrome
  native_tree:
    id: R3
    text: "The native accessibility tree exposes pane controls and file rows through real XCUI queries."
    kind: functional
    risk: high
    verify: apps/workbench/macos/Scripts/run-native-e2e.sh
---
flowchart TD
    r1[R1 independent terminal controls] --> workbenchmacuitests_testpanetoolbarremainsincontentchrome[WorkbenchMacUITests.testPaneToolbarRemainsInContentChrome]
    r2[R2 bounded file rows] --> workbenchmacuitests_testfilesauxiliarycolumnshowsfixtureentries[WorkbenchMacUITests.testFilesAuxiliaryColumnShowsFixtureEntries]
    r3[R3 native tree] --> apps_workbench_macos_scripts_run_native_e2e_sh[apps/workbench/macos/Scripts/run-native-e2e.sh]
```
