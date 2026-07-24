---
id: '2493'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-two-pane-contract
entry: profile-menu
nodes:
  profile-menu: { kind: start, label: "profile menu selection" }
  idle: { kind: process, label: "idle terminal session" }
  pane: { kind: process, label: "focused or right pane" }
  start: { kind: terminal, label: "explicit start" }
edges:
  - { from: profile-menu, to: idle }
  - { from: idle, to: pane }
  - { from: pane, to: start }
---
flowchart LR
  menu([Profile menu]) --> idle[Idle session]
  idle --> pane[Focused / right pane]
  pane --> start([Explicit PTY start])
```

A `ProjectTerminalWorkspace` contains session tabs plus ordered pane records. Each pane references zero or one session tab. The first profile choice fills the selected empty pane. A second profile choice adds the right pane when capacity permits. A pane close terminates only its running session and leaves one empty pane when it was the last pane.

The view renders a project-qualified pane layout and keeps inactive project renderers mounted but noninteractive. Pane focus is model state. The profile menu and pane actions provide visible labels and keyboard-focusable controls; state dots convey lifecycle without a redundant text status.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/workbench/macos/Sources/WorkbenchMacCore/WorkbenchModel.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchModel
  - path: apps/workbench/macos/Sources/WorkbenchMac/WorkbenchView.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchView
  - path: apps/workbench/macos/Tests/WorkbenchMacCoreTests/WorkbenchModelTests.swift
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: WorkbenchModelTests
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
id: workbench-two-pane-contract-verification
requirements:
  idle_menu:
    id: R2
    text: "Profile menu creates idle sessions without a PTY request."
    kind: regression
    risk: medium
    verify: WorkbenchModelTests.testProfileMenuCreatesIdlePaneSession
  native_package:
    id: R3
    text: "Native package builds and passes regression tests."
    kind: regression
    risk: low
    verify: swift test --package-path apps/workbench/macos
  pane_state:
    id: R1
    text: "Projects retain independent one-or-two-pane state and focus."
    kind: functional
    risk: medium
    verify: WorkbenchModelTests.testProjectsRestoreIndependentPaneWorkspaces
---
flowchart TD
    r1[R1 pane state] --> workbenchmodeltests_testprojectsrestoreindependentpaneworkspaces[WorkbenchModelTests.testProjectsRestoreIndependentPaneWorkspaces]
    r2[R2 idle menu] --> workbenchmodeltests_testprofilemenucreatesidlepanesession[WorkbenchModelTests.testProfileMenuCreatesIdlePaneSession]
    r3[R3 native package] --> swift_test_package_path_apps_workbench_macos[swift test --package-path apps/workbench/macos]
```
