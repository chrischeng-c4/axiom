---
change: spec-consolidation
group: spec-consolidation-enforcement
date: 2026-03-23
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| crates/cclab-sdd/logic/reference-context.md | lifecycle-enforcement | high | Central router for reference_context CRR lifecycle, Create prompt template — where spec_plan and spec listing are injected, spec_plan array generation (action, main_spec_ref, sections), BUG: pre_filter_specs only lists root-level files — misses subdirectories |
| crates/cclab-sdd/logic/change-merge.md | lifecycle-enforcement | high | Merge reads main_spec_ref from change spec frontmatter, Writes to cclab/specs/{main_spec_ref} — create or overwrite, merge_strategy field references (dead code to remove), Add merge-time validation: reject root-level main_spec_ref |
| crates/cclab-sdd/logic/change-spec.md | lifecycle-enforcement | high | Change spec preparation from spec_plan, action: modify copies existing spec, action: create writes skeleton, merge_strategy references to remove |
| crates/cclab-sdd/logic/state-machine.md | state-flow | high | Phase flow — reference_context phase where spec_plan is generated, Phase routing for validation gates |
| crates/cclab-sdd/interfaces/tools/workflow-tools.md | interfaces | high | sdd_workflow_create_reference_context tool definition — interface affected by spec listing format change |
| crates/cclab-sdd/interfaces/tools/artifact-tools.md | interfaces | high | create-reference-context artifact schema, merge_strategy param to remove from artifact tools |
| crates/cclab-sdd/interfaces/cli/commands.md | interfaces | high | CLI command tree — scaffold-spec and validate-spec-structure commands to add |
| crates/cclab-sdd/logic/scope-resolution.md | structure | high | Layer 1 scope resolution via config.toml, pre_filter_specs function — generates In-Scope Specs listing for agent prompt, BUG: uses read_dir (flat) not recursive — only sees root-level .md files, Must be updated to reflect recursive listing behavior |
| crates/cclab-sdd/tools/utils/validate-spec-completeness.md | validation | medium | Existing spec validation pattern — validate-spec-structure builds on this |
| crates/cclab-sdd/tools/utils/validate-change.md | validation | medium | Hard error validation pattern — merge-time validation follows this model |
| crates/cclab-sdd/config/agents.md | config | low | Agent config — reference for prompt injection points |
| crates/cclab-sdd/README.md | architecture | low | Overall SDD architecture overview |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| spec-structure-rules | create | crates/cclab-sdd/logic/spec-structure.md | overview, schema, logic, cli |
| reference-context-update | modify | crates/cclab-sdd/logic/reference-context.md | changes |
| change-merge-update | modify | crates/cclab-sdd/logic/change-merge.md | changes |
| change-spec-update | modify | crates/cclab-sdd/logic/change-spec.md | changes |
| cli-commands-update | modify | crates/cclab-sdd/interfaces/cli/commands.md | changes |
| artifact-tools-update | modify | crates/cclab-sdd/interfaces/tools/artifact-tools.md | changes |
| scope-resolution-update | modify | crates/cclab-sdd/logic/scope-resolution.md | changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: spec-consolidation

**Verdict**: APPROVED

### Summary

Reference context is comprehensive and well-structured. All 3 requirement layers (structure, lifecycle enforcement, dead code removal) are covered by spec_plan entries. The previous revision addressed the missing scope-resolution-update entry and the workflow-tools key requirement error. Two minor relevance-scoring inconsistencies remain (state-machine.md and workflow-tools.md rated high without spec_plan entries) but these do not materially affect spec authoring — the spec_plan itself is correct and complete.

### Checklist

- ✅ All affected crates/areas from pre-clarifications are covered by at least one spec
  - Structure layer: spec-structure-rules (create) + cli-commands-update. Lifecycle layer: reference-context-update, change-merge-update, scope-resolution-update. Dead code layer: change-spec-update, change-merge-update, artifact-tools-update. All covered.
- ✅ Relevance scores are reasonable (high = directly implements, medium = related, low = background)
  - Minor: state-machine.md and workflow-tools.md are rated high but have no spec_plan entries (context-only, arguably should be medium). Does not affect spec authoring since spec_plan is the actionable output.
- ✅ Key requirements listed per spec are accurate
  - Verified all 12 specs against actual content. reference-context.md lists BUG note for pre_filter_specs which is technically owned by scope-resolution.md, but the prompt template in reference-context.md consumes the output, so the cross-reference is acceptable.
- ✅ No irrelevant specs included
  - All 12 specs are relevant. 2 low-relevance (agents, README) provide legitimate background.
- ✅ spec_plan: every entry has main_spec_ref set (not null)
  - All 7 entries have valid main_spec_ref paths
- ✅ spec_plan: sections are reasonable for the requirements
  - spec-structure-rules: overview+schema+logic+cli (appropriate for new validation rules spec). All 6 modify entries: changes section only.
- ✅ spec_plan: modify entries have valid source paths
  - All 6 modify targets verified to exist under cclab/specs/: reference-context.md, change-merge.md, change-spec.md, commands.md, artifact-tools.md, scope-resolution.md

### Issues

No issues found.
