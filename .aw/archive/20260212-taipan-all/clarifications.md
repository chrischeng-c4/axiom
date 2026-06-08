---
change: taipan-all
date: 2026-02-12
---

# Clarifications

## Q1: Git Workflow
- **Question**: Which git workflow to use for this change?
- **Answer**: in_place — work directly on the current feat/taipan branch
- **Rationale**: The feat/taipan branch already exists and is the natural place for all taipan work

## Q2: Implementation Order
- **Question**: Any priority ordering preference for the 67 issues?
- **Answer**: Bottom-up: Lexer (205-212) → Parser (213-235) → Pattern matching (235-239) → Types (240-249) → Config (250-251) → Build (252-254) → FFI (255-271)
- **Rationale**: Dependency order ensures each layer builds on the previous one — lexer tokens feed parser, parser AST feeds type checker, etc.

## Q3: Scope
- **Question**: All 67 issues in one change?
- **Answer**: Yes, a single unified change covering issues #205 through #271
- **Rationale**: User explicitly requested 'one change, all issues' to implement the full taipan compiler stack

