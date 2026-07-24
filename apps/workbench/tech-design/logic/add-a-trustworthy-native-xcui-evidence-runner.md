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
