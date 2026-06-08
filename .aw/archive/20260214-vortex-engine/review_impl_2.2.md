---
verdict: REVIEWED
file: implementation
iteration: 1
task_id: 2.2
---

# Review: implementation:task_2.2 (Iteration 1)

**Change ID**: vortex-engine

## Summary

Task 2.2 FAIL (partial implementation): R1 is implemented, but R2 and R3 are not fully satisfied by the current ECS source.

## Issues

- **[high]** R3 (Parallel System Execution) is not implemented. `Schedule::run` executes systems strictly sequentially and there is no access-set declaration, conflict detection (read/write or write/write), stage boundary planning, or parallel execution path.
  - *Recommendation*: Introduce system access metadata (reads/writes + stage), build an execution plan that groups only non-conflicting systems for parallel execution, and enforce deterministic stage ordering before running groups.
- **[high]** R2 (Query System composition) is not fully implemented. The query module only exposes fixed arity helpers (`query_one`, `query_two`, `query_three`) and lacks a composable query spec with `with`, `without`, and optional predicate filters.
  - *Recommendation*: Add a query-spec API (or builder) supporting `with`, `without`, and predicates, and execute it using smallest-cardinality driving sets to avoid full-world scans while preserving ergonomic iteration.
- **[medium]** Tests do not cover required conflict-aware scheduling behavior. Existing ECS tests validate sequential schedule execution only.
  - *Recommendation*: Add tests for scheduler conflict planning (RW/WW conflicts serialized, disjoint systems parallelized) and deterministic stage boundary enforcement.

## Verdict

- [ ] APPROVED
- [x] REVIEWED
- [ ] REJECTED

