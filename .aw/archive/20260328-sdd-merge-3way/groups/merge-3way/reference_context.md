---
change: sdd-merge-3way
group: merge-3way
date: 2026-03-27
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| crates/cclab-sdd/logic/change-merge.md | merge-logic | high | 3-way merge using git merge-file: ours=current main spec, base=.base.md snapshot, theirs=cleaned change spec, Conflict handling: abort entire merge if any spec has conflicts; report conflicted specs to user for manual resolution, Fallback to current overwrite behavior when no .base.md present (action:create or legacy changes), find_specs_to_merge() must skip .base.md files — they are merge artifacts, not spec content, Audit log must distinguish 3way-merge, create, overwrite actions |
| crates/cclab-sdd/logic/change-spec.md | spec-preparation | high | prepare_modify_spec() in spec_plan.rs must save {spec_id}.base.md snapshot alongside the working spec in groups/{group}/specs/, Snapshot is a copy of cclab/specs/{main_spec_ref} taken at spec preparation time (action:modify only), action:create specs do not need a base snapshot |
| crates/cclab-sdd/logic/state-machine.md | state-machine | medium | ChangeMergeCreated → ChangeArchived: the terminal merge phase where 3-way merge executes |
| crates/cclab-sdd/logic/change-spec-logic.md | spec-preparation | medium | find_specs_to_merge() already iterates groups/*/specs/ — .base.md files must be excluded from that iteration |
| crates/cclab-sdd/skills/merge.md | skills | low | Merge skill invocation context — user triggers cclab sdd workflow create-change-merge |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| merge-3way | modify | crates/cclab-sdd/logic/change-merge.md | overview, logic, changes |
| spec-prep-base-snapshot | modify | crates/cclab-sdd/logic/change-spec.md | overview, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: sdd-merge-3way

**Verdict**: APPROVED

### Summary

All 5 referenced specs are relevant. Both spec plan entries are well-formed with valid main_spec_ref paths under logic/ subfolders. All four pre-clarification requirements (Q1–Q4) are covered: base snapshot → change-spec.md, git merge-file + conflict handling → change-merge.md, .base.md skip → change-merge.md key requirements. Two low-severity notes: (1) 'audit log must distinguish 3way-merge, create, overwrite actions' is extrapolated from pre-clarifications but is grounded in the existing create_change_merge.rs implementation (audit_log already tracks create/overwrite at lines 85, 135–136); (2) change-spec-logic.md has main_spec_ref:~ (stale change spec in main specs dir) — pre-existing issue, does not affect this change's correctness.

### Checklist

- ✅ All affected crates/areas from pre-clarifications are covered by at least one spec
  - Q1 (prepare_modify_spec base snapshot) → change-spec.md; Q2 (git merge-file in create_change_merge.rs) → change-merge.md; Q3 (abort on conflict) → change-merge.md; Q4 (find_specs_to_merge skip .base.md) → change-merge.md key requirements + change-spec-logic.md reference
- ✅ Relevance scores are reasonable (high = directly implements, medium = related, low = background)
  - change-merge.md=high (core merge logic changes), change-spec.md=high (spec preparation step), state-machine.md=medium (phase background), change-spec-logic.md=medium (documents find_specs_to_merge function), skills/merge.md=low (invocation context only) — all accurate
- ✅ Key requirements listed per spec are accurate (match actual requirement IDs)
  - All key requirements match Q1–Q4 answers. 'Audit log must distinguish 3way-merge, create, overwrite actions' is an extrapolation not in pre-clarifications but is grounded in the existing audit_log implementation (create_change_merge.rs lines 85, 135–136 already track create/overwrite).
- ✅ No irrelevant specs included
  - All 5 specs are directly or contextually relevant to the 3-way merge change
- ✅ spec_plan: every entry has main_spec_ref set (not null)
  - merge-3way → crates/cclab-sdd/logic/change-merge.md; spec-prep-base-snapshot → crates/cclab-sdd/logic/change-spec.md — both set
- ✅ spec_plan: sections are reasonable for the requirements
  - merge-3way: [overview, logic, changes] appropriate for significant flowchart change + file list; spec-prep-base-snapshot: [overview, changes] sufficient for a targeted one-fs-write addition to prepare_modify_spec()
- ✅ spec_plan: modify entries have valid source paths
  - Both source specs verified to exist: cclab/specs/crates/cclab-sdd/logic/change-merge.md and cclab/specs/crates/cclab-sdd/logic/change-spec.md
- ✅ spec_plan: main_spec_ref paths include a subfolder (not root-level under crate)
  - Both paths are crates/cclab-sdd/logic/... — logic/ subfolder present
- ✅ spec_plan: each spec file covers exactly one logical unit (not multiple unrelated concerns)
  - change-merge.md = Change Merge logic; change-spec.md = Change Spec lifecycle — distinct units, no overlap
- ✅ spec_plan: no spec file would require duplicate section types (split into separate files if needed)
  - merge-3way sections [overview, logic, changes] — no duplicates; spec-prep-base-snapshot sections [overview, changes] — no duplicates
- ✅ spec_plan: spec paths mirror source structure (interfaces/, logic/, generate/)
  - Both under logic/ — correct since only implementation logic changes, no interface layer affected

### Issues

- **[LOW]** change-spec-logic.md (cclab/specs/crates/cclab-sdd/logic/change-spec-logic.md) has main_spec_ref:~ — this is a stale/incomplete change spec that was placed in main specs directory without finalization. Many sections are TODO stubs.
  - *Recommendation*: Pre-existing quality issue with that file. No action needed for this change. Using it as reference context is acceptable.
- **[LOW]** 'Audit log must distinguish 3way-merge, create, overwrite actions' appears in change-merge.md key requirements but is not grounded in Q1–Q4 pre-clarifications. However, it is grounded in the existing implementation: create_change_merge.rs already has an audit_log (line 85) tracking 'create'/'overwrite' actions (lines 135–136). Adding '3way-merge' is a natural extension.
  - *Recommendation*: Acceptable extrapolation. No revision needed.
