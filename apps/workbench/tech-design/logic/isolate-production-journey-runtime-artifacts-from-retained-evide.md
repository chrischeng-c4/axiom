---
id: '2502'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-test-artifact-routing
entry: test
nodes:
  test: { kind: start, label: "test run" }
  runtime: { kind: terminal, label: "ignored runtime artifacts" }
  refresh: { kind: process, label: "explicit refresh" }
  retained: { kind: terminal, label: "tracked evidence" }
edges:
  - { from: test, to: runtime }
  - { from: refresh, to: retained }
---
flowchart LR
  test[Test run] --> runtime[Ignored runtime artifacts]
  refresh[Explicit refresh] --> retained[Tracked evidence]
```

Routine tests write run-specific screenshots, transcripts, and measurements below repository-local ignored runtime state. The existing tracked evidence remains the retained review baseline and is updated only through an explicit refresh environment switch.

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
