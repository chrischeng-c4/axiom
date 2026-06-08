---
change: 197
date: 2026-02-12
---

# Clarifications

## Q1: Change Type
- **Question**: Is this a spec-only change or does it also need code changes?
- **Answer**: Spec-only. Add error recovery documentation sections to existing spec files.
- **Rationale**: Issue #197 explicitly asks for adding documentation sections, not implementing error recovery logic.

## Q2: Git Workflow
- **Question**: Which git workflow should be used?
- **Answer**: in_place — work on the current main branch alongside other genesis consistency fixes.
- **Rationale**: All 12 issues (#193-#204) use in_place workflow on the same branch.

## Q3: Target Files
- **Question**: Which spec files need the error recovery section?
- **Answer**: delegate-agent.md and run-change/README.md as specified in the issue. These are the two specs that cover agent delegation and workflow orchestration.
- **Rationale**: Issue body explicitly names these two files.

## Q4: Recovery Scenarios
- **Question**: Which error recovery scenarios should be covered?
- **Answer**: All 5 from the issue: (1) agent tool call failure, (2) genesis_agent verification failed, (3) cyclic dependency fallback, (4) partial state recovery, (5) concurrent STATE.yaml access.
- **Rationale**: Cover all scenarios mentioned in the issue.

