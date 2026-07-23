---
id: '2461'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Contract
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-native-switch-contract
entry: user_action
nodes:
  user_action: { kind: start, label: select-project-or-tab }
  project_state: { kind: process, label: selected-project-workspace }
  tab_state: { kind: process, label: selected-terminal-layer }
  no_sidecar: { kind: terminal, label: no-project-lifecycle-call }
  same_renderer: { kind: terminal, label: same-renderer-on-return }
edges:
  - { from: user_action, to: project_state, label: project }
  - { from: project_state, to: no_sidecar }
  - { from: user_action, to: tab_state, label: tab }
  - { from: tab_state, to: same_renderer }
---
flowchart LR
  user_action([User selection]) -->|Project| project_state[Selected project workspace]
  project_state --> no_sidecar([No existing PTY mutation])
  user_action -->|Tab| tab_state[Selected terminal layer]
  tab_state --> same_renderer([Same renderer on return])
```

A project selection is a synchronous presentation-state transition: the selected project id, launch folder, and visible file listing describe the same registered project. It affects only future launches; active and exited terminal tabs retain their own process state and cwd.

A tab selection is also a presentation-state transition. It must not launch, terminate, resize, or rebuild a terminal process. Each launched tab keeps one native SwiftTerm view and coordinator for its lifetime; only the selected layer accepts input and is visible.
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
