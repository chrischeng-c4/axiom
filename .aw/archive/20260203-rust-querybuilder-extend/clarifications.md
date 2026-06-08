---
change: rust-querybuilder-extend
date: 2026-01-31
---

# Clarifications

## Q1: Method Chaining
- **Question**: How should we handle method chaining in Rust QueryBuilder?
- **Answer**: Mutable self (&mut self) pattern - more efficient and suitable for PyO3
- **Rationale**: Using &mut self is more efficient as it avoids cloning, and PyO3 bindings work well with mutable references. This differs from Python's clone pattern but provides better performance.

## Q2: Feature Scope
- **Question**: Should we include Window functions and CTEs in this phase?
- **Answer**: Full feature parity - include Window functions, CTEs, Subqueries, Set operations
- **Rationale**: Implementing all features at once ensures complete parity with Python and avoids partial migration issues. The Rust crate already has some of these structures in place.

## Q3: Git Workflow
- **Question**: What's your preferred git workflow for this change?
- **Answer**: In place - stay on current branch
- **Rationale**: Simple workflow for focused changes, no branch management overhead.

