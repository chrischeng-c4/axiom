---
change: sdd-p1
date: 2026-02-23
issues: [463, 464, 465, 467, 468, 469, 470, 471]
---

# Context Clarifications

## Q1: General
- **Question**: Which git workflow should be used?
- **Answer**: in_place on sdd-and-mamba branch.
- **Rationale**: 

## Q2: General
- **Question**: Are #463, #464, #465 already fixed?
- **Answer**: Yes, implemented in commit 0e5936c. Out of scope — only #467-#471 are active.
- **Rationale**: 

## Q3: General
- **Question**: Which verdict label to standardize on?
- **Answer**: APPROVED. Update all impl prompts (gap_codebase_spec.rs, gap_codebase_knowledge.rs, gap_spec_knowledge.rs, proposal.rs, implement.rs) from PASS → APPROVED.
- **Rationale**: 

## Q4: General
- **Question**: How to fix revise action labels?
- **Answer**: Change action="create" → action="revise" in clarify.rs:360 and proposal.rs:171.
- **Rationale**: 

## Q5: General
- **Question**: How to align review checklists?
- **Answer**: Bring impl checklists up to spec completeness. Gap analysis: add action_needed, repair_action, type enum checks (6 items). Proposal: add DAG validity, gap coverage, context_refs, spec coherence, orphan specs, impact scope, scope_areas (7 items). Post-clarifications: add resolution quality, consistency checks (5 items). Explore context: fix three-way inconsistencies. Spec context: keep extra codebase_paths/knowledge_refs item.
- **Rationale**: 

## Q6: General
- **Question**: What fields to add to gap analysis create prompts?
- **Answer**: action_needed (bool), repair_action (string), type enum (code_without_spec, spec_without_code, convention_violation, pattern_mismatch, undocumented_pattern, spec_contradicts_knowledge), relevance map step.
- **Rationale**: 

## Q7: General
- **Question**: DAG context loop routing strategy?
- **Answer**: Always route to explore_spec + explore_knowledge only. Remove explore_codebase. No complexity-based branching needed.
- **Rationale**: 

## Q8: General
- **Question**: Which modules are in scope?
- **Answer**: crate cclab-sdd: src/mcp/tools/run_change/ (prompts), src/orchestrator/ (DAG/reviews). Active issues: #467, #468, #469, #470, #471 only.
- **Rationale**: 

## Dependency Graph

| Order | Issue | Depends On |
|-------|-------|------------|
| 1 | #463 — bug(sdd): sdd_run_change skips spec creation phase after proposal_approved | — |
| 2 | #464 — feat(cli): cclab init — platform selection with auth method options | — |
| 3 | #465 — feat(sdd): fetch_issues — add glab (GitLab) CLI support | — |
| 4 | #467 — SDD: Verdict label inconsistency — APPROVED vs PASS across all review prompts | — |
| 5 | #468 — SDD: Revise actions use action="create" instead of action="revise" in write_artifact calls | — |
| 6 | #469 — SDD: Review checklist items missing or mismatched across all phases | — |
| 7 | #470 — SDD: Gap analysis prompts missing action_needed, repair_action, and type enum values | — |
| 8 | #471 — SDD: DAG context loop ignores complexity — always routes to explore_codebase | — |

```mermaid
graph LR
    463["#463 bug(sdd): sdd_run_change skips spec creation phase after proposal_approved"]
    464["#464 feat(cli): cclab init — platform selection with auth method options"]
    465["#465 feat(sdd): fetch_issues — add glab (GitLab) CLI support"]
    467["#467 SDD: Verdict label inconsistency — APPROVED vs PASS across all review prompts"]
    468["#468 SDD: Revise actions use action="create" instead of action="revise" in write_artifact calls"]
    469["#469 SDD: Review checklist items missing or mismatched across all phases"]
    470["#470 SDD: Gap analysis prompts missing action_needed, repair_action, and type enum values"]
    471["#471 SDD: DAG context loop ignores complexity — always routes to explore_codebase"]
```

