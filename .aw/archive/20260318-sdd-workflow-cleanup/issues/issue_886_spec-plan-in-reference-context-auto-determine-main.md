---
number: 886
title: "Spec Plan in reference_context: auto-determine main_spec_ref for change-spec"
state: open
labels: [enhancement, P1, crate:sdd]
group: "spec-plan"
---

# #886 — Spec Plan in reference_context: auto-determine main_spec_ref for change-spec

## Summary

Add a structured **Spec Plan** section (YAML codeblock) to `reference_context.md` so that `create-change-spec` can auto-determine `main_spec_ref` and `merge_strategy` instead of relying on the agent to guess.

## Problem

Currently `generate_skeleton()` is always called with `main_spec_ref=None`. The agent must figure out during the analyze step whether to modify an existing spec or create a new one. This is error-prone — agents may guess wrong or skip setting `main_spec_ref` entirely.

## Proposed Format

In `reference_context.md`, after the specs table:

~~~markdown
# Spec Plan

```yaml
specs:
  - id: patterns-update
    action: modify
    main_spec_ref: cclab-mamba/parser/patterns.md
    merge_strategy: extend
    description: "Add AS patterns (R7), **rest mapping (R8)"
  - id: hir-patterns
    action: create_new
    description: "HirPattern node definitions, HirMatch statement"
```
~~~

Program parses this → `create-change-spec` skeleton generation uses `main_spec_ref` directly:
- `action: modify` → `generate_skeleton(main_spec_ref=path)` copies existing spec
- `action: create_new` → `generate_skeleton(main_spec_ref=None)` empty skeleton

## Files to Change

### Specs (2)
- `cclab/specs/cclab-sdd/logic/reference-context.md` — add Spec Plan format definition
- `cclab/specs/cclab-sdd/logic/change-spec.md` — update Mode 1/2 to read from spec plan

### Codebase (5)
- `common_reference_context.rs` — `render_specs_markdown()` append Spec Plan YAML
- `create_reference_context.rs` — update agent prompt to produce spec plan
- `workflow/helpers.rs` — extend `SpecInfo` struct with `main_spec_ref`, `merge_strategy`
- `common_change_spec.rs` — add `parse_spec_plan_from_reference_context()`
- `create_change_spec.rs` — skeleton generation reads spec plan; update analyze prompt

## Benefits

- Deterministic: program decides modify vs create_new, not the agent
- Skeleton has correct base content from the start
- Agent only needs to fill/modify sections, not figure out which spec to target
- Backward compatible: if no spec plan, falls back to current behavior (agent determines)
