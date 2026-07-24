---
id: '2507'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-native-xcui-evidence-runner
entry: runner
nodes:
  runner: { kind: start, label: "native E2E runner" }
  artifacts: { kind: process, label: "unique ignored artifact directory" }
  build: { kind: process, label: "build sidecar and Xcode test products" }
  execute: { kind: process, label: "run XCUI with resultBundlePath" }
  validate: { kind: decision, label: "complete result with tests and zero failures" }
  fail: { kind: terminal, label: "fail and preserve evidence" }
  pass: { kind: terminal, label: "print counts and paths" }
  screenshot: { kind: process, label: "named keepAlways screenshot" }
edges:
  - { from: runner, to: artifacts }
  - { from: artifacts, to: build }
  - { from: build, to: execute }
  - { from: execute, to: validate }
  - { from: validate, to: fail, label: "missing or failed" }
  - { from: validate, to: pass, label: "executed > 0 and failures = 0" }
  - { from: screenshot, to: execute, label: "xcresult attachment" }
---
flowchart TD
    runner([Native E2E runner]) --> artifacts[Create unique ignored artifact directory]
    artifacts --> build[Build sidecar and Xcode UI-test products]
    build --> execute[Run XCUI with explicit resultBundlePath]
    execute --> validate{Complete result; tests > 0; failures = 0?}
    validate -->|no| fail([Fail and preserve evidence])
    validate -->|yes| pass([Print counts and artifact paths])
    screenshot[Named keepAlways screenshot] -->|xcresult attachment| execute
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/workbench/macos/Scripts/run-native-e2e.sh
    action: create
    section: logic
    impl_mode: hand-written
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
id: workbench-native-xcui-evidence-runner-verification
requirements:
  named_screenshot_evidence:
    id: R3
    text: "XCUI journeys attach stable keepAlways screenshots that remain inspectable in the verified result bundle."
    kind: regression
    risk: medium
    verify: WorkbenchMacUITests.testPaneToolbarRemainsInContentChrome
  runner_executes_native_suite:
    id: R1
    text: "One native runner builds the bundled sidecar and executes the Workbench macOS XCUI target with an explicit unique result bundle."
    kind: functional
    risk: high
    verify: apps/workbench/macos/Scripts/run-native-e2e.sh
  runner_rejects_false_green:
    id: R2
    text: "The runner returns success only when xcresult reports at least one executed test and zero failed tests."
    kind: regression
    risk: high
    verify: apps/workbench/macos/Scripts/run-native-e2e.sh
---
flowchart TD
    r1[R1 runner executes native suite] --> apps_workbench_macos_scripts_run_native_e2e_sh[apps/workbench/macos/Scripts/run-native-e2e.sh]
    r2[R2 runner rejects false green] --> apps_workbench_macos_scripts_run_native_e2e_sh
    r3[R3 named screenshot evidence] --> workbenchmacuitests_testpanetoolbarremainsincontentchrome[WorkbenchMacUITests.testPaneToolbarRemainsInContentChrome]
```
