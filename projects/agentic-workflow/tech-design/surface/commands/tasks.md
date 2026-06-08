---
id: score-tasks
fill_sections: [schema, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "Command/root TDs support CLI workflow chain routing and root-runner dispatch."
---

# TasksCommands Type

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TasksCommands:
    type: string
    enum: [Generate, Create]
    description: Available subcommands for `score tasks`.
    x-rust-enum:
      derive: [Subcommand]
      variants:
        - name: Generate
          kind: struct
          doc: "Auto-generate tasks from specs (recommended)."
          fields:
            - { name: change_id, rust_type: String }
        - name: Create
          kind: struct
          doc: "Create tasks file from JSON file (legacy, for manual override)."
          fields:
            - { name: change_id, rust_type: String }
            - { name: json_file, rust_type: PathBuf, "x-clap-arg": "long" }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/tasks.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - TasksCommands
    description: |
      Codegen replaces the enum declaration only.
  - path: projects/agentic-workflow/src/cli/tasks.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, imports, the
      `run` function and JSON parsing logic.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Subcommand enum with struct variants, one with variant-field clap-arg.
- [schema] x-clap-arg on json_file field of Create variant.
- [changes] Standard split.
