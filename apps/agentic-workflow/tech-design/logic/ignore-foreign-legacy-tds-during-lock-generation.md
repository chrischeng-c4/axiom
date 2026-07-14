---
id: '1705'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: td-gen-spec-resolution-contract
entry: requested
nodes:
  requested:
    kind: start
    label: "td gen work-item request"
  explicit:
    kind: decision
    label: "Explicit or project-qualified TD spec available?"
  active:
    kind: process
    label: "Use the active configured spec and its project lock"
  legacy:
    kind: decision
    label: "A unique legacy candidate is configured?"
  fallback:
    kind: terminal
    label: "Use configured legacy fallback only"
  error:
    kind: terminal
    label: "Require an explicit spec path"
edges:
  - { from: requested, to: explicit }
  - { from: explicit, to: active, label: "yes" }
  - { from: explicit, to: legacy, label: "no" }
  - { from: legacy, to: fallback, label: "yes" }
  - { from: legacy, to: error, label: "no" }
---
flowchart TD
    requested([td gen request]) --> explicit{configured or explicit spec?}
    explicit -->|yes| active[use active project spec + lock]
    explicit -->|no| legacy{unique configured legacy candidate?}
    legacy -->|yes| fallback([configured fallback])
    legacy -->|no| error([request explicit spec])
```

`td gen` resolves an explicit or project-qualified active spec before any
legacy worktree discovery. Legacy fallback is valid only for a unique candidate
under a configured project root. A foreign `.aw` path is never selected and is
never passed into TD lock validation.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tech-design/surface/interfaces/src/td.md
    action: modify
    section: logic
    impl_mode: codegen
  - path: apps/agentic-workflow/src/cli/td.rs
    action: modify
    section: logic
    impl_mode: codegen
  - path: apps/agentic-workflow/tech-design/surface/validate/tests/td_no_merge_test.md
    action: modify
    section: unit-test
    impl_mode: codegen
  - path: apps/agentic-workflow/tests/cli/tests/td_no_merge_test.rs
    action: modify
    section: unit-test
    impl_mode: codegen
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: td-gen-configured-root-isolation-verification
requirements:
  foreign_legacy_isolation:
    id: R1
    text: "td gen for a configured active project ignores a foreign unconfigured legacy TD candidate and proceeds to its active spec lock."
    kind: functional
    risk: medium
    verify: cargo test -p agentic-workflow --test cli_tests test_td_gen_ignores_foreign_unconfigured_legacy_spec -- --nocapture
---
flowchart TD
    r1[R1 foreign legacy isolation] --> cargo_test_p_agentic_workflow_test_cli_tests_test_td_gen_ignores_foreign_unconfigured_legacy_spec_nocapture[cargo test -p agentic-workflow --test cli_tests test_td_gen_ignores_foreign_unconfigured_legacy_spec -- --nocapture]
```
