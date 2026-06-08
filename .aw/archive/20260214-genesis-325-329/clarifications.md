---
change: genesis-325-329
date: 2026-02-14
---

# Clarifications

## Q1: Implementation Scope
- **Question**: Should we do #325 (SpecIR contract) first and defer the rest, or attempt all 5 issues in one change?
- **Answer**: All 5 issues in one change: SpecIR contract → migrate generators → Aurora cleanup → Prism unify → Genesis integrate
- **Rationale**: User wants end-to-end delivery of the full spec-to-code pipeline refactor in a single coordinated change.

## Q2: SpecIR Location
- **Question**: Should SpecIR be a new shared crate or defined inside cclab-aurora?
- **Answer**: In cclab-aurora. SpecIR lives in Aurora since Aurora owns spec formats. Prism already depends on Aurora.
- **Rationale**: No new crate needed. Aurora owns the spec format layer, and Prism already has cclab-aurora as a dependency, so the SpecIR types flow naturally.

## Q3: Git Workflow
- **Question**: Which git workflow for this change?
- **Answer**: in_place — work on current sdd branch
- **Rationale**: User is already on the sdd branch and prefers to continue working there.

## Q4: Affected Crates
- **Question**: Which crates/paths will this change affect?
- **Answer**: cclab-aurora (SpecIR definition, remove generators), cclab-prism (absorb generators, unify codegen pipeline), cclab-genesis (implement phase integration with Prism codegen)
- **Rationale**: Three crates involved: Aurora scopes down, Prism absorbs generators and unifies, Genesis wires up the implement phase.

## Q5: Execution Order
- **Question**: What is the dependency order for the 5 issues?
- **Answer**: #325 (SpecIR contract) → #326 (migrate generators) + #328 (Prism unify) in parallel → #327 (Aurora cleanup) → #329 (Genesis integrate)
- **Rationale**: SpecIR contract must exist before migration can happen. Aurora cleanup depends on migration being complete. Genesis integration depends on both SpecIR and Prism unification.

