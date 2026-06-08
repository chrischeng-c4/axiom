---
change: genesis-186-28
date: 2026-02-14
---

# Clarifications

## Q1: Scope adjustment
- **Question**: Issue #28 is already fixed. Should we proceed with only #186?
- **Answer**: Yes, #28 is confirmed fixed and closed. Only #186 remains.
- **Rationale**: #28's fix (include_str!() compile-time embedding) was already implemented. Verified all templates have {{project_path}} substitution.

## Q2: Git workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place — work on current sdd branch
- **Rationale**: User prefers working on the current branch directly.

## Q3: #186 scope
- **Question**: All 4 improvements or partial?
- **Answer**: All 4: LLM enrichment, review cycle, validator integration, diagram generation. But evaluate each for current relevance since #186 is an older issue.
- **Rationale**: User wants full implementation but acknowledges some features may have evolved since the issue was filed.

## Q4: Backward compatibility
- **Question**: Should --quick flag be supported?
- **Answer**: Yes, issue specifies --quick flag to skip LLM enrichment and keep current fast behavior.
- **Rationale**: Ensures backward compatibility for large codebases where fast AST-only mode is preferred.

