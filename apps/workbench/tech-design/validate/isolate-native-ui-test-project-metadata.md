---
id: '2505'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-native-ui-test-state-root-contract
entry: resolve
nodes:
  resolve: { kind: start, label: "resolve launch state root" }
  fixture: { kind: decision, label: "fixture and absolute override?" }
  test: { kind: terminal, label: "repo-local UI-test root" }
  profile: { kind: terminal, label: "profile state root" }
  services: { kind: process, label: "projects + runtime + log" }
edges:
  - { from: resolve, to: fixture }
  - { from: fixture, to: test, label: "yes" }
  - { from: fixture, to: profile, label: "no" }
  - { from: test, to: services }
  - { from: profile, to: services }
---
flowchart LR
  resolve([Resolve state root]) --> fixture{Fixture and absolute override?}
  fixture -->|yes| test[Repo-local UI-test root]
  fixture -->|no| profile[Stable / Beta root]
  test --> services[ProjectStore + runtime + log]
  profile --> services
```

`WorkbenchRuntimeProfile.resolvedStateRoot(environment:fileManager:)` returns `WORKBENCH_UI_TEST_STATE_ROOT` only when `WORKBENCH_UI_TEST_FOLDER` is also nonempty and the override is an absolute file URL. Otherwise it returns the existing profile state root. This prevents a stray environment variable or relative path from redirecting a human launch.

`WorkbenchMacApp` resolves the root once, then derives ProjectStore storage, `runtime/`, and `logs/workbench.log` from that same value. `WorkbenchDiagnosticLog` accepts the resolved log URL directly so test launches do not write into either real profile. The XCUI launcher computes the repository root from `#filePath`, assigns `.axiom-workbench/test-artifacts/ui-tests/<run-id>`, and removes only that run directory during teardown. The root remains ignored and inspectable if a test aborts before teardown.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/workbench/macos/Sources/WorkbenchMacCore/WorkbenchRuntimeProfile.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchRuntimeProfile
  - path: apps/workbench/macos/Sources/WorkbenchMacCore/DiagnosticLog.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchDiagnosticLog
  - path: apps/workbench/macos/Sources/WorkbenchMac/WorkbenchMacApp.swift
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: WorkbenchMacApp
  - path: apps/workbench/macos/Tests/WorkbenchMacCoreTests/WorkbenchRuntimeProfileTests.swift
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: WorkbenchRuntimeProfileTests
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
id: workbench-native-ui-test-state-isolation-verification
requirements:
  native_suite:
    id: R4
    text: "The native Swift package compiles and executes all state isolation regressions."
    kind: regression
    risk: medium
    verify: swift test --package-path apps/workbench/macos
  normal_profiles:
    id: R2
    text: "A normal stable or beta launch ignores the UI-test override and retains its profile-specific state root."
    kind: regression
    risk: high
    verify: WorkbenchRuntimeProfileTests.testNormalLaunchDoesNotAcceptUITestStateOverride
  override:
    id: R1
    text: "A fixture launch accepts an absolute UI-test state root and routes native application state there."
    kind: functional
    risk: high
    verify: WorkbenchRuntimeProfileTests.testUITestStateRootRequiresFixtureAndAbsoluteOverride
  xcui_wiring:
    id: R3
    text: "XCUI launches supply one isolated repository-local root and continue to complete the pane interaction journey."
    kind: regression
    risk: medium
    verify: WorkbenchMacUITests.testPaneToolbarOffersAddAndExplicitSplitActions
---
flowchart TD
    r1[R1 override] --> workbenchruntimeprofiletests_testuiteststaterootrequiresfixtureandabsoluteoverride[WorkbenchRuntimeProfileTests.testUITestStateRootRequiresFixtureAndAbsoluteOverride]
    r2[R2 normal profiles] --> workbenchruntimeprofiletests_testnormallaunchdoesnotacceptuiteststateoverride[WorkbenchRuntimeProfileTests.testNormalLaunchDoesNotAcceptUITestStateOverride]
    r3[R3 xcui wiring] --> workbenchmacuitests_testpanetoolbaroffersaddandexplicitsplitactions[WorkbenchMacUITests.testPaneToolbarOffersAddAndExplicitSplitActions]
    r4[R4 native suite] --> swift_test_package_path_apps_workbench_macos[swift test --package-path apps/workbench/macos]
```
