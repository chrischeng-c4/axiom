---
id: sdd-tools-task-type
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# TaskType Enum

## Overview
<!-- type: overview lang: markdown -->

The `TaskType` enum in `projects/agentic-workflow/src/tools/task.rs` enumerates the
agent task kinds the `get_task` MCP tool can dispatch. Single enum,
9 unit variants, no serde traits, no rename_all — variants stay in
PascalCase both in Rust and as the canonical wire string used by the
hand-written `from_str` dispatch.

- `TaskType` — variants Explore, CreateSpec, ReviewSpec, ReviseSpec,
  ReviseTasks, Implement, CodeReview, Resolve, ReviewArchive.
- Derives `[Debug, Clone, Copy, PartialEq, Eq]`. NO Serialize, NO
  Deserialize, NO Default.

This spec exercises the small no-serde enum pattern already used in
`sdd-models-section`. The custom `from_str` dispatch maps wire strings
(snake_case: "explore", "create_spec", ...) to PascalCase variants —
that mapping is implemented hand-written because it does not match
the standard `std::str::FromStr` trait shape (returns `Result<Self>`
i.e. `anyhow::Result`, not `Result<Self, Self::Err>`).

Hand-written outside CODEGEN: module docstring, `use` statements
(`super::{get_optional_string, get_required_string, ToolDefinition}`,
`crate::Result`, `chrono::Local`, `serde_json::{json, Value}`,
`std::collections::HashMap`, `std::path::Path`),
`impl TaskType { fn from_str(s: &str) -> Result<Self> }`, `pub fn definition()`,
`pub fn execute(...)`, and any helper functions.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TaskType:
    type: string
    enum:
      - Explore
      - CreateSpec
      - ReviewSpec
      - ReviseSpec
      - ReviseTasks
      - Implement
      - CodeReview
      - Resolve
      - ReviewArchive
    description: >
      Agent task kind dispatched by the get_task MCP tool. Variants stay
      PascalCase both in Rust and as the wire string consumed by the
      hand-written from_str dispatch.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/task.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - TaskType
    description: |
      Codegen replaces the `TaskType` enum declaration only.
  - path: projects/agentic-workflow/src/tools/task.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, use statements,
      `impl TaskType { fn from_str(s: &str) -> Result<Self> }`,
      `pub fn definition()`, `pub fn execute(...)`, and any helper
      functions. The custom `from_str` does not match the standard
      `std::str::FromStr` trait shape (returns `anyhow::Result`), so it
      stays hand-written.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Accurately describes single-enum migration: 9 variant names enumerated, derive list matches source `[Debug, Clone, Copy, PartialEq, Eq]`, hand-written `from_str` rationale (anyhow::Result vs std::str::FromStr) is correct.
- [schema] Schema is complete and correct: `x-rust-enum.derive` matches source, no `serde_rename_all` (preserves PascalCase wire strings used by hand-written from_str), all 9 variants enumerated. No ambiguity for the generator.
- [changes] Two-entry split correctly partitions codegen (`TaskType` declaration only via `replaces:`) from hand-written (impl block, free fns, helpers). Sufficient for `aw td gen-code` to place CODEGEN markers.
