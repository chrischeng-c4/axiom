---
id: '2502'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-test-artifact-routing-contract
entry: test
nodes:
  test: { kind: start, label: "routine test" }
  runtime: { kind: terminal, label: "ignored runtime artifacts" }
  refresh: { kind: process, label: "WORKBENCH_REFRESH_EVIDENCE=1" }
  retained: { kind: terminal, label: "tracked evidence baseline" }
edges:
  - { from: test, to: runtime }
  - { from: refresh, to: retained }
---
flowchart LR
  test[Routine test] --> runtime[.axiom-workbench/test-artifacts]
  refresh[Explicit refresh] --> retained[Tracked evidence baseline]
```

Both Rust and browser production journeys resolve one artifact root. The default is `<repo>/.axiom-workbench/test-artifacts/production-journey/v1`; `WORKBENCH_REFRESH_EVIDENCE=1` selects the retained `apps/workbench/evidence/production-journey/v1` path. The runtime root is repository-ignored. Assertions and manifest schema remain unchanged.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .gitignore
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: repository ignore rules
  - path: apps/workbench/tests/production_journey.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: evidence_root
  - path: apps/workbench/e2e/production-journey.spec.js
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: evidenceDir
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: workbench-runtime-artifact-isolation-verification
requirements:
  retained:
    id: R2
    text: "Tracked retained evidence is not changed by routine verification."
    kind: regression
    risk: high
    verify: git diff --exit-code -- apps/workbench/evidence/production-journey/v1
  runtime:
    id: R1
    text: "Routine production journeys write run-specific artifacts below ignored repository runtime state."
    kind: regression
    risk: high
    verify: cargo test -p workbench --test production_journey -- --nocapture
---
flowchart TD
    r1[R1 runtime] --> cargo_test_p_workbench_test_production_journey_nocapture[cargo test -p workbench --test production_journey -- --nocapture]
    r2[R2 retained] --> git_diff_exit_code_apps_workbench_evidence_production_journey_v1[git diff --exit-code -- apps/workbench/evidence/production-journey/v1]
```
