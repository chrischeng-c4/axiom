---
id: '1679'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: terminal-code-check-exact-spec-evidence-scope
entry: code_check
nodes:
  code_check: { kind: start, label: "aw td code-check <slug>" }
  resolve: { kind: process, label: "use Issue.implements exact TD paths" }
  promise: { kind: process, label: "extract only target hand-written create/modify entries" }
  unrelated: { kind: process, label: "ignore unrelated project TD entries" }
  evidence: { kind: decision, label: "target entries changed since target Td-Init?" }
  error: { kind: terminal, label: "refuse with target-path diagnostics" }
  done: { kind: terminal, label: "complete terminal code-check" }
edges:
  - { from: code_check, to: resolve }
  - { from: resolve, to: promise }
  - { from: unrelated, to: promise, label: "excluded" }
  - { from: promise, to: evidence }
  - { from: evidence, to: error, label: "no" }
  - { from: evidence, to: done, label: "yes" }
---
flowchart TD
  code_check([code-check slug]) --> resolve[exact Issue.implements paths]
  unrelated[unrelated project TD] -. excluded .-> promise[target hand-written paths]
  resolve --> promise
  promise --> evidence{all target diffs exist?}
  evidence -->|no| error([target-only refusal])
  evidence -->|yes| done([complete lifecycle])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tech-design/surface/validate/tests/td_no_merge_test.md
    action: modify
    section: source
    impl_mode: hand-written
    description: Add the source snapshot regression that proves a terminal code-check reads only the requested WI's declared TD paths.
  - path: apps/agentic-workflow/tests/cli/tests/td_no_merge_test.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: Describe the real-AW integration regression without claiming whole-file ownership; the td_no_merge_test source snapshot is the sole CODEGEN owner.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: terminal-code-check-exact-spec-evidence-scope-verification
requirements:
  failure_diagnostics:
    id: R2
    text: "A target evidence refusal names only paths declared by the completing WI and never paths from another project's TD."
    kind: regression
    risk: high
    verify: cargo test -p agentic-workflow --test cli_tests test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec -- --nocapture
  target_spec_only:
    id: R1
    text: "A completed hand-written WI closes even when a different project's TD declares unimplemented hand-written paths, because terminal evidence comes only from the completing WI's Issue.implements path."
    kind: regression
    risk: high
    verify: cargo test -p agentic-workflow --test cli_tests test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec -- --nocapture
---
flowchart TD
    r1[R1 target spec only] --> cargo_test_p_agentic_workflow_test_cli_tests_test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec_nocapture[cargo test -p agentic-workflow --test cli_tests test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec -- --nocapture]
    r2[R2 failure diagnostics] --> cargo_test_p_agentic_workflow_test_cli_tests_test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec_nocapture
```
