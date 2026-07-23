---
id: '2461'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-switching-state
entry: project_select
nodes:
  project_select: { kind: start, label: select-project }
  workspace: { kind: process, label: update-selected-workspace }
  files: { kind: process, label: refresh-files-listing }
  no_pty_change: { kind: terminal, label: preserve-existing-pty }
  tab_select: { kind: start, label: select-tab }
  show_layer: { kind: process, label: show-tab-layer }
  retained_view: { kind: process, label: retain-terminal-view }
  incremental: { kind: terminal, label: feed-new-bytes-only }
edges:
  - { from: project_select, to: workspace }
  - { from: workspace, to: files }
  - { from: files, to: no_pty_change }
  - { from: tab_select, to: show_layer }
  - { from: show_layer, to: retained_view }
  - { from: retained_view, to: incremental }
---
flowchart LR
  project_select([Select project]) --> workspace[Update selected workspace]
  workspace --> files[Refresh Files and launch root]
  files --> no_pty_change([Preserve existing PTY])
  tab_select([Select tab]) --> show_layer[Show selected terminal layer]
  show_layer --> retained_view[Retain tab-keyed TerminalView]
  retained_view --> incremental([Feed new bytes only])
```

Project selection updates the selected project id, launch root, and file listing together on the main actor. This selection is for future terminal launches; it never sends a lifecycle request or rewrites an existing tab's cwd.

Every non-idle and non-failed tab owns a mounted `TerminalSurface` keyed by tab id. Inactive layers are visually hidden and non-interactive instead of destroyed. Their coordinators retain both the SwiftTerm terminal buffer and fed-byte cursor, so a tab switch only exposes its existing renderer; polling supplies subsequent incremental bytes.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/workbench/macos/Sources/WorkbenchMacCore/WorkbenchModel.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchModel
    description: Publish one selected project workspace value so project id, launch root, and Files listing change together without changing existing PTY tabs.
  - path: apps/workbench/macos/Sources/WorkbenchMac/WorkbenchView.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchView
    description: Keep launched terminal surfaces mounted in tab-keyed layers and expose only the selected layer.
  - path: apps/workbench/macos/Tests/WorkbenchMacCoreTests/WorkbenchModelTests.swift
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: WorkbenchModelTests
    description: Prove project selection changes launch and files state without issuing terminal lifecycle requests or modifying existing tabs.
  - path: apps/workbench/macos/UITests/WorkbenchMacUITests.swift
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: WorkbenchMacUITests
    description: Prove a running shell tab retains terminal content after switching to another tab and back.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: workbench-switching-state-verification
requirements:
  project_workspace_state:
    id: R1
    text: "Selecting a registered project updates selected project, launch root, and Files listing together."
    kind: functional
    risk: medium
    verify: WorkbenchModelTests.projectsSwitchWorkspaceStateWithoutTouchingTerminals
  pty_cwd_preservation:
    id: R3
    text: "Project selection does not alter existing PTY cwd or lifecycle."
    kind: regression
    risk: high
    verify: WorkbenchModelTests.projectsSwitchWorkspaceStateWithoutTouchingTerminals
  terminal_renderer_retention:
    id: R2
    text: "A running terminal retains its native renderer and does not replay its transcript after tab selection changes."
    kind: regression
    risk: high
    verify: WorkbenchMacUITests.testNativeShellJourney
---
flowchart TD
    r1[R1 project workspace state] --> workbenchmodeltests_projectsswitchworkspacestatewithouttouchingterminals[WorkbenchModelTests.projectsSwitchWorkspaceStateWithoutTouchingTerminals]
    r3[R3 pty cwd preservation] --> workbenchmodeltests_projectsswitchworkspacestatewithouttouchingterminals
    r2[R2 terminal renderer retention] --> workbenchmacuitests_testnativeshelljourney[WorkbenchMacUITests.testNativeShellJourney]
```
