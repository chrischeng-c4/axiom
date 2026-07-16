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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-llm-task-topics-verification
requirements:
  compact_outline:
    id: R1
    text: "Default outline exposes the stable core, services, container, k8s, and guide topic IDs without rendering the full guide."
    kind: regression
    risk: medium
    verify: vat_cli_convention::cli_convention_llm_topics_are_task_scoped
  offline_topic_contract:
    id: R2
    text: "Each advertised topic resolves offline and names its primary runnable VAT commands."
    kind: functional
    risk: high
    verify: vat_cli_convention::cli_convention_llm_topics_are_task_scoped
---
flowchart TD
    r1[R1 compact outline] --> vat_cli_convention_cli_convention_llm_topics_are_task_scoped[vat_cli_convention::cli_convention_llm_topics_are_task_scoped]
    r2[R2 offline topic contract] --> vat_cli_convention_cli_convention_llm_topics_are_task_scoped
```
