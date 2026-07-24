---
id: '2505'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-ui-test-state-isolation
entry: launch
nodes:
  launch: { kind: start, label: "native app launch" }
  test: { kind: process, label: "XCUI fixture environment" }
  local: { kind: terminal, label: "repo-local ignored test state" }
  profile: { kind: terminal, label: "stable or beta profile state" }
edges:
  - { from: launch, to: test, label: "fixture present" }
  - { from: test, to: local }
  - { from: launch, to: profile, label: "normal launch" }
---
flowchart LR
  launch([Native app launch]) -->|XCUI fixture| test[UI-test environment]
  test --> local[.axiom-workbench/test-artifacts/ui-tests]
  launch -->|normal| profile[Stable / Beta state root]
```

This design applies only to native app bootstrap when `WORKBENCH_UI_TEST_FOLDER` is present. The XCUI launcher supplies an explicit `WORKBENCH_UI_TEST_STATE_ROOT` below the repository-local ignored test-artifact directory. ProjectStore, LocalRuntimeServer, and diagnostic logging use that root for the test process. Normal stable and beta launches continue using their existing profile roots.
