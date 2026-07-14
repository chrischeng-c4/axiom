---
id: '1696'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: project-td-before-legacy-fallback
entry: read
nodes:
  read: { kind: start, label: "terminal code-check reads WI" }
  implements: { kind: decision, label: "implements has Markdown TD paths?" }
  declared: { kind: process, label: "use declared WI TD paths" }
  default: { kind: process, label: "derive project-qualified TD from label and slug" }
  default_found: { kind: decision, label: "project TD resolves?" }
  project_td: { kind: process, label: "use project TD path" }
  legacy: { kind: process, label: "discover unique legacy worktree TD" }
  scoped: { kind: process, label: "collect changes only from scoped TD paths" }
  verify: { kind: terminal, label: "validate evidence only for scoped paths" }
edges:
  - { from: read, to: implements }
  - { from: implements, to: declared, label: "yes" }
  - { from: implements, to: default, label: "no" }
  - { from: default, to: default_found }
  - { from: default_found, to: project_td, label: "yes" }
  - { from: default_found, to: legacy, label: "no" }
  - { from: declared, to: scoped }
  - { from: project_td, to: scoped }
  - { from: legacy, to: scoped }
  - { from: scoped, to: verify }
---
flowchart TD
  read([read WI]) --> implements{implements TD?}
  implements -->|yes| declared[declared WI TD]
  implements -->|no| default[derive project TD]
  default --> default_found{project TD resolves?}
  default_found -->|yes| project_td[project TD]
  default_found -->|no| legacy[unique legacy TD]
  declared --> scoped[scoped changes]
  project_td --> scoped
  legacy --> scoped
  scoped --> verify([verify scoped evidence])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tech-design/surface/interfaces/src/cb.md
    action: modify
    section: source
    impl_mode: hand-written
  - path: apps/agentic-workflow/src/cli/cb.rs
    action: modify
    section: source
    impl_mode: codegen
  - path: apps/agentic-workflow/tech-design/surface/validate/tests/td_no_merge_test.md
    action: modify
    section: source
    impl_mode: hand-written
  - path: apps/agentic-workflow/tests/cli/tests/td_no_merge_test.rs
    action: modify
    section: source
    impl_mode: codegen
```
