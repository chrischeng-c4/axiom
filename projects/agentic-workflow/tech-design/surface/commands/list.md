---
id: score-list
fill_sections: [schema, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "Command/root TDs support CLI workflow chain routing and root-runner dispatch."
---

# List Output Types

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ActiveItem:
    type: object
    required: [slug, phase, branch, title]
    description: |
      Active change item: a change with a worktree and known phase.
    properties:
      slug:
        type: string
        description: "Change slug."
      phase:
        type: string
        description: "Current phase from the nested issue file. \"unknown\" when unreadable."
      branch:
        type: string
        description: "Branch name. Always cclab/<slug>."
      title:
        type: string
        description: "Issue title."
    x-rust-struct:
      derive: [Serialize, Debug, Clone, PartialEq]

  IdleItem:
    type: object
    required: [slug, state, title]
    description: |
      Idle issue item: an issue without a matching worktree.
    properties:
      slug:
        type: string
        description: "Issue slug."
      state:
        type: string
        description: "Issue state. \"open\" or \"draft\"."
      title:
        type: string
        description: "Issue title."
    x-rust-struct:
      derive: [Serialize, Debug, Clone, PartialEq]

  ListOutput:
    type: object
    required: [active, idle]
    description: |
      Aggregated output from `score list`.
    properties:
      active:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ActiveItem>"
        description: "Active items."
      idle:
        type: array
        items: { type: object }
        x-rust-type: "Vec<IdleItem>"
        description: "Idle items."
    x-rust-struct:
      derive: [Serialize, Debug, Default]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/list.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ActiveItem
      - IdleItem
      - ListOutput
    description: |
      Codegen replaces all three struct declarations.
  - path: projects/agentic-workflow/src/cli/list.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, imports, the
      `run_dual_source` function and all helpers.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Three pure data carriers; standard Serialize derive.
- [schema] All well-formed; Vec<T> via x-rust-type for ListOutput fields.
- [changes] Standard split with all three structs in `replaces`.
