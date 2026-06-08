---
verdict: REVIEWED
file: implementation
iteration: 1
task_id: 2.1
---

# Review: implementation:task_2.1 (Iteration 1)

**Change ID**: 191

## Summary

Task 2.1 is largely implemented (block_plus module, validator, generator, and wiring exist; targeted block_plus tests pass), but there are correctness and validation gaps against the block-plus spec, especially around shape rendering and nested validation semantics.

## Checklist

- ✅ Task 2.1 files exist and are wired into module exports and MCP handlers/tools
- ✅ Targeted block_plus unit tests execute successfully
- ❌ Implementation fully satisfies nested frontmatter validation semantics in spec R2
- ❌ Generated Mermaid syntax is robust for all declared shapes

## Issues

- **[high]** Nested block span validation is incomplete. Validation only checks top-level `diagram.blocks` widths against `diagram.columns`, but does not recursively validate child block widths against `child_columns` (or inferred nested columns). This allows invalid nested layouts to pass validation, contrary to the spec’s frontmatter validation requirement for nested blocks.
  - *Recommendation*: Add recursive width/column validation in `BlockValidator` for all nested `children`, enforcing width <= effective nested column count and ensuring nested column count is >= 1.
- **[high]** Subroutine shape rendering injects an embedded newline in `format_shape`, which can generate malformed Mermaid lines (especially when width suffix `:N` is appended). This can break diagram syntax for valid inputs using `subroutine` shape.
  - *Recommendation*: Remove the newline from `BlockShape::Subroutine` formatting and keep all line breaks handled only by the caller (`generate_blocks`). Add tests covering `subroutine` with and without `width > 1`.
- **[medium]** Hexagon and diamond shapes are rendered with the same Mermaid token, so `hexagon` does not produce a distinct shape. This violates expected shape fidelity for block syntax support.
  - *Recommendation*: Update `format_shape` to emit the correct Mermaid representation for `hexagon` (distinct from `diamond`) and add assertion-based shape snapshot tests for each supported shape.

## Verdict

- [ ] APPROVED
- [x] REVIEWED
- [ ] REJECTED

