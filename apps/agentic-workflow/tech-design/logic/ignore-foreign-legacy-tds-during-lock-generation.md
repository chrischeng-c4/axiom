---
id: '1705'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: configured-td-lock-discovery
entry: candidates
nodes:
  candidates:
    kind: start
    label: "TD lock discovery candidates"
  configured:
    kind: decision
    label: "Candidate is active spec or under a configured project td_path?"
  include:
    kind: process
    label: "Validate or generate the configured TD lock"
  ignore:
    kind: terminal
    label: "Ignore foreign unconfigured legacy TD"
edges:
  - { from: candidates, to: configured }
  - { from: configured, to: include, label: "yes" }
  - { from: configured, to: ignore, label: "no" }
---
flowchart TD
    candidates([lock candidates]) --> configured{active or configured td_path?}
    configured -->|yes| include[validate/generate lock]
    configured -->|no| ignore([foreign legacy ignored])
```

TD generation considers the requested active spec and TDs under configured
project roots only. A worktree-local legacy `.aw` file outside every configured
`td_path` is preservation input, not a lock participant, and cannot block an
unrelated project's generation.

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
