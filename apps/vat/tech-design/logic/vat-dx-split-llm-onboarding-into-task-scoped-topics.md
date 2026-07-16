---
id: '1818'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-llm-task-topics
entry: llm
nodes:
  outline: { kind: start, label: "vat llm defaults to a compact topic inventory" }
  registry: { kind: process, label: "one registry maps stable topic IDs to concise task contracts" }
  guide: { kind: process, label: "legacy guide remains the complete backward-compatible reference" }
  done: { kind: terminal, label: "agents select only the documentation needed for the current task" }
edges:
  - { from: outline, to: registry }
  - { from: registry, to: guide }
  - { from: guide, to: done }
---
```

`vat llm` keeps its default `outline` compact and derives it solely from the `cli_std::llm::Topic` registry. The registry adds stable `core`, `services`, `container`, and `k8s` task topics while retaining `guide` as the full compatibility document. Each concise topic names its runnable commands and explicit boundaries; it does not duplicate the complete guide. The JSON outline exposes every stable ID, so regression tests can lock the task inventory and prove each topic resolves offline.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/src/commands/llm.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: TOPICS
  - path: apps/vat/tests/vat_cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: cli_convention_llm_flags
```
