---
change: sdd-impl-test-split
group: sdd-impl-test-split
date: 2026-03-21
---

# Requirements

Three targeted fixes to the SDD implementation workflow:

1. **create_change_impl.rs — Split dispatch into two phases**:
   - Phase 1: implement code only → verify build passes (`cargo build`)
   - Phase 2: implement tests only → verify test count in diff matches the test-plan count declared in the spec
   - Currently, a single dispatch handles code + tests together with no build or test-count verification

2. **review_change_impl.rs — Add hard checklist enforcement**:
   - Extend the review prompt with a hard checklist: (a) code matches requirements, (b) tests match test-plan, (c) tests pass
   - Hard REJECT rule: if spec defines a `## Test Plan` section but the diff contains no tests, the reviewer MUST return verdict `REJECTED` — no exceptions
   - Currently the review prompt has soft verdict guidelines with no test-plan enforcement

3. **create_change_spec.rs — Guard `create_complete` on failure**:
   - Do NOT set `create_complete: true` in frontmatter when `failed_sections` is non-empty
   - Instead, return an error response so mainthread can retry
   - Currently (lines 519-524), `create_complete: true` is unconditionally written even after section fills fail
