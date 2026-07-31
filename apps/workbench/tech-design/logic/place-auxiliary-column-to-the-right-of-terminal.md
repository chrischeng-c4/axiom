---
id: '2459'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-auxiliary-right-contract
entry: selected-project
nodes:
  selected-project: { kind: start, label: "selected registered project" }
  profile: { kind: decision, label: "beta runtime profile?" }
  terminal: { kind: process, label: "render central terminal workspace" }
  files: { kind: process, label: "render read-only Files auxiliary" }
  stable: { kind: terminal, label: "Projects | Terminal" }
  beta: { kind: terminal, label: "Projects | Terminal | Auxiliary" }
edges:
  - { from: selected-project, to: profile }
  - { from: profile, to: stable, label: "stable" }
  - { from: profile, to: terminal, label: "beta" }
  - { from: terminal, to: files }
  - { from: files, to: beta }
---
flowchart LR
    selectedProject([Selected project]) --> profile{Beta profile?}
    profile -->|Stable| stable([Projects | Terminal])
    profile -->|Beta| terminal[Central terminal]
    terminal --> files[Read-only Files]
    files --> beta([Projects | Terminal | Auxiliary])
```

The user-visible layout contract is fixed: Projects is the native NavigationSplitView sidebar, Terminal is the primary flexible detail region, and Beta-only Auxiliary is the trailing bounded detail region. `terminalWorkspace` must precede `auxiliaryColumn` in the detail HStack, separated by exactly one divider. Stable omits the trailing divider and Auxiliary column.

The file listing stays read-only and project-scoped. The layout reorder does not initiate a process, alter an existing terminal's cwd, change the active tab, or affect the native window sidebar toggle.
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
id: workbench-auxiliary-right-contract-verification
requirements:
  beta_layout_order:
    id: R1
    text: "The Beta detail hierarchy places terminal controls before the accessible Auxiliary Files column."
    kind: e2e
    risk: medium
    verify: WorkbenchMacUITests.testFilesAuxiliaryColumnFollowsTerminalWorkspace
  stable_no_auxiliary:
    id: R2
    text: "The Stable profile does not mount the Beta-only Auxiliary column."
    kind: regression
    risk: medium
    verify: WorkbenchRuntimeProfileTests.testStableAndBetaProductsAreDistinct
---
flowchart TD
    r1[R1 beta layout order] --> workbenchmacuitests_testfilesauxiliarycolumnfollowsterminalworkspace[WorkbenchMacUITests.testFilesAuxiliaryColumnFollowsTerminalWorkspace]
    r2[R2 stable no auxiliary] --> workbenchruntimeprofiletests_teststableandbetaproductsaredistinct[WorkbenchRuntimeProfileTests.testStableAndBetaProductsAreDistinct]
```
