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
  resolve: { kind: process, label: "resolve exact Issue.implements TD spec paths" }
  target_evidence: { kind: process, label: "collect hand-written create/modify paths only from resolved specs" }
  unrelated_spec: { kind: process, label: "unrelated project TD with incomplete paths" }
  complete: { kind: decision, label: "every target path has a Td-Init-to-HEAD diff?" }
  refuse: { kind: terminal, label: "error names only target paths" }
  close: { kind: terminal, label: "complete terminal lifecycle" }
edges:
  - { from: code_check, to: resolve }
  - { from: resolve, to: target_evidence }
  - { from: unrelated_spec, to: target_evidence, label: "must not contribute" }
  - { from: target_evidence, to: complete }
  - { from: complete, to: refuse, label: "no" }
  - { from: complete, to: close, label: "yes" }
---
flowchart TD
  code_check([aw td code-check]) --> resolve[resolve Issue.implements]
  unrelated_spec[unrelated Mamba TD] -. ignored .-> target_evidence[target spec evidence only]
  resolve --> target_evidence
  target_evidence --> complete{every target diff present?}
  complete -->|no| refuse([error: target paths only])
  complete -->|yes| close([terminal lifecycle closes])
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
    impl_mode: codegen
    description: Regenerate the real-AW integration regression from the updated source snapshot.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: terminal-code-check-exact-spec-evidence-scope-verification
requirements:
  failure_diagnostics:
    id: R2
    text: "When the completing WI lacks evidence, its refusal names its own declared paths and never names another project's TD paths."
    kind: regression
    risk: high
    verify: cargo test -p agentic-workflow --test cli_tests test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec -- --nocapture
  target_spec_only:
    id: R1
    text: "A completed hand-written WI must close even when a different project's TD declares unimplemented hand-written paths, because terminal evidence is resolved only from the completing WI's Issue.implements path."
    kind: regression
    risk: high
    verify: cargo test -p agentic-workflow --test cli_tests test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec -- --nocapture
---
flowchart TD
    r1[R1 target spec only] --> cargo_test_p_agentic_workflow_test_cli_tests_test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec_nocapture[cargo test -p agentic-workflow --test cli_tests test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec -- --nocapture]
    r2[R2 failure diagnostics] --> cargo_test_p_agentic_workflow_test_cli_tests_test_code_check_ignores_unrelated_hand_written_evidence_outside_wi_spec_nocapture
```
