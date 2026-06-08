---
change: main-spec-awareness
date: 2026-02-02
---

# Clarifications

## Q1: List Output
- **Question**: When listing main specs, should we include spec metadata (title, spec_type, requirements count) or just file paths?
- **Answer**: With metadata - Include title, spec_type, spec_group from frontmatter for better context
- **Rationale**: Richer metadata helps the planner understand existing specs without needing to read each one individually, reducing API calls and context usage

## Q2: Merge Types
- **Question**: For merge_strategy field, which strategies should we support?
- **Answer**: extend, replace, patch, new - Add patch for partial updates
- **Rationale**: Patch strategy provides finer control for incremental updates, useful when only adding specific requirements or scenarios without affecting the rest of the spec

## Q3: Enforcement
- **Question**: Should the planner be required to check main specs, or just encouraged?
- **Answer**: Required - Prompt explicitly requires calling list_main_specs before creating proposal
- **Rationale**: Mandatory checking prevents duplicate specs and ensures consistency with existing architecture, which is the main goal of this change

## Q4: Migration
- **Question**: How should we handle the existing target_crate field during migration?
- **Answer**: Replace immediately - Rename target_crate to spec_group in all existing specs
- **Rationale**: Clean break is better than carrying legacy field names; spec_group is language-agnostic and clearer

