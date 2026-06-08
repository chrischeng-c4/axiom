---
id: score-issues-top
fill_sections: [schema, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "Command/root TDs support CLI workflow chain routing and root-runner dispatch."
---

# Work-Item CLI Top-Level Args

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  IssuesArgs:
    type: object
    required: [command]
    description: Top-level args for `aw wi`.
    properties:
      command:
        type: object
        x-rust-type: "IssuesCommand"
        x-clap-command: "subcommand"
        description: "The selected subcommand."
    x-rust-struct:
      derive: [Debug, Args]

  IssuesCommand:
    type: string
    enum: [List, Show, Create, Update, Close, Find, Enrich, Validate, FillSection, Review, Arbitrate, Prune]
    description: Available subcommands for `aw wi`.
    x-rust-enum:
      derive: [Debug, Subcommand]
      variants:
        - name: List
          kind: tuple
          doc: "List open work-items from the configured backend."
          fields:
            - { rust_type: ListArgs }
        - name: Show
          kind: tuple
          doc: "Show a single work-item by slug or numeric id."
          fields:
            - { rust_type: ShowArgs }
        - name: Create
          kind: tuple
          doc: "Create a new work-item."
          fields:
            - { rust_type: CreateArgs }
        - name: Update
          kind: tuple
          doc: "Update an existing work-item's metadata or body."
          fields:
            - { rust_type: UpdateArgs }
        - name: Close
          kind: tuple
          doc: "Close a work-item, optionally with a reason."
          fields:
            - { rust_type: CloseArgs }
        - name: Find
          kind: tuple
          doc: "Search work-items by text query."
          fields:
            - { rust_type: FindArgs }
        - name: Enrich
          kind: tuple
          doc: "Fill the Reference Context section via agent exploration."
          fields:
            - { rust_type: EnrichArgs }
        - name: Validate
          kind: tuple
          doc: "Validate work-item quality (CRR gate)."
          fields:
            - { rust_type: ValidateArgs }
        - name: FillSection
          kind: tuple
          doc: "Fill work-item sections via structured round-trip."
          fields:
            - { rust_type: FillSectionArgs }
        - name: Review
          kind: tuple
          doc: "Review the filled work-item via reviewer round-trip."
          fields:
            - { rust_type: ReviewArgs }
        - name: Arbitrate
          kind: tuple
          doc: "Arbitrate a stalled CRRR loop after second needs-revision."
          fields:
            - { rust_type: ArbitrateArgs }
        - name: Prune
          kind: tuple
          doc: "Remove leftover worktrees whose work-items are already closed."
          fields:
            - { rust_type: PruneArgs }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/issues.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - IssuesArgs
      - IssuesCommand
    description: |
      Codegen replaces only the top-level Args and Subcommand enum.
  - path: projects/agentic-workflow/src/cli/issues.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module imports, all 14 other
      Args/Subcommand structs (ListArgs, ShowArgs, CreateArgs,
      UpdateArgs, CloseArgs, FindArgs, EnrichArgs, ValidateArgs,
      FillSectionArgs, ReviewArgs, ArbitrateArgs,
      PruneArgs, StateFilter, TypeFilter), all
      run_* dispatch functions, helper functions, and tests.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Top-level Args + Subcommand wrapper; 19 nested arg/filter types stay hand-written.
- [schema] Subcommand enum with 16 tuple variants; x-clap-command "subcommand" on IssuesArgs.command.
- [changes] Standard split with only top 2 in `replaces`; rest preserved.
