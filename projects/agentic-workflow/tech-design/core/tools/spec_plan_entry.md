---
id: sdd-tools-spec-plan-entry
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# SpecPlanEntry

## Overview
<!-- type: overview lang: markdown -->

Single struct `SpecPlanEntry` in `projects/agentic-workflow/src/tools/spec_plan.rs`.
Carries one row of the spec-planning table found in issue Reference
Context bodies — `(spec_id, action, main_spec_ref, source, sections)`.
Plain data shape with partial-derives `[Debug, Clone, Serialize,
Deserialize]`.

Hand-written outside CODEGEN: module preamble, all `use` statements,
all free fns / helpers, and the `#[cfg(test)] mod tests` block.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SpecPlanEntry:
    type: object
    required: [spec_id, action, main_spec_ref, sections]
    description: One row of the spec-planning table in an issue's Reference Context.
    properties:
      spec_id:
        type: string
        description: "Spec identifier slug."
      action:
        type: string
        description: "Action keyword (create / update / merge / ...)."
      main_spec_ref:
        type: string
        description: "Reference to the main spec this entry plans."
      source:
        type: string
        description: "Optional source artifact (e.g. issue slug)."
        x-serde-default: true
      sections:
        type: array
        items: { type: string }
        description: "Section names this spec covers."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize", "serde::Deserialize"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/spec_plan.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - SpecPlanEntry
    description: |
      Codegen replaces the SpecPlanEntry struct declaration only.
  - path: projects/agentic-workflow/src/tools/spec_plan.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module preamble, all `use`
      statements, all free fns / helpers, and the `#[cfg(test)] mod tests`
      block.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Single-struct scope correctly identified. Hand-written boundary explicit.
- [schema] Partial-derive list, Vec-in-required convention, Option auto-wrap on `source` all match the source.
- [changes] Two-entry split clean.
