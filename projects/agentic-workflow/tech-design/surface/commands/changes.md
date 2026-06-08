---
id: score-changes
fill_sections: [schema, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "Command/root TDs support CLI workflow chain routing and root-runner dispatch."
---

# ChangesArgs + ChangesCommand Types

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ChangesArgs:
    type: object
    required: [command]
    description: Args wrapper for `score changes`.
    properties:
      command:
        type: object
        x-rust-type: "ChangesCommand"
        x-clap-command: "subcommand"
        description: "The selected subcommand."
    x-rust-struct:
      derive: [Debug, Args]

  ChangesCommand:
    type: string
    enum: [FilesAffected]
    description: Available subcommands for `score changes`.
    x-rust-enum:
      derive: [Debug, Subcommand]
      variants:
        - name: FilesAffected
          kind: struct
          doc: "List files that a change will modify (from spec Changes section)."
          fields:
            - { name: change_id, rust_type: String }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/changes.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ChangesArgs
      - ChangesCommand
    description: |
      Codegen replaces both type declarations.
  - path: projects/agentic-workflow/src/cli/changes.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, imports, the `run`
      function, and all helpers + tests.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Args wrapper + Subcommand enum, the standard clap pattern.
- [schema] x-clap-command "subcommand" on ChangesArgs.command; struct-variant on the enum side.
- [changes] Standard split.
