# 2023-patrol-handoff

**Issue:** #2023 — bug(mamba): tuple-swap unpack `a, b = b, a + b` produces garbage at runtime
**Branch:** `issue-bug-tuple-swap-unpack-a-b-b-a-b-produces-garb` (in-place workflow; no td-* branch)
**Phase:** `td_merged` (GitHub state: CLOSED)
**Spec:** `.score/tech_design/projects/mamba/specs/mamba-bug-tuple-unpack-evaluation-order.md`

## Problem

Multi-target tuple-unpack `a, b = b, a + b` (and n>=3 rotations like `a, b, c = c, a, b`) silently writes garbage / uninitialised memory at runtime — the JIT backend crashes outright on the n=2 swap. Root cause: the `Assign` lowering reuses LHS storage in-place while still reading from it, so the RHS expression `a + b` sees the post-mutation value of `a` instead of the pre-assignment one.

## Findings

CRRR drove the issue from `draft -> open` (R1-R8 fill-section ladder, reviewer #1 approved) and then drove the TD from `td_inited -> td_created -> td_reviewed -> td_merged` in three patrol ticks. The approved TD spec is the source of truth for the fix:

- **Dependency** (classDiagram, 7 types): `TupleUnpackLowering` owns `RhsTempBuffer` that materialises every RHS expression before any LHS slot is written; `LengthCheck` raises `ValueError` on mismatch; `ConformanceTest` asserts on the lowering.
- **Logic** (flowchart): `is_destructure` decision routes n=1 to the unchanged single-target path (R6 regression guard); n>=2 falls through `eval_all_rhs` (source-order) -> `len_check` -> `copy_targets` or `raise_value`. No back-edge to in-place corruption.
- **Changes** (4 files):
  - modify `projects/mamba/src/lowering/assign.rs` — materialise RHS before LHS write
  - modify `projects/mamba/src/runtime/unpack.rs` — stack-allocated small-N (n<=8) RhsTempBuffer to preserve R5 zero-heap-alloc invariant for the swap idiom
  - create `projects/mamba/tests/conformance/tuple_unpack_eval_order.rs` — R1/R2/R3/R7 coverage
  - create `projects/mamba/tests/conformance/fixtures/tuple_unpack_eval_order.py` — CPython 3.12 parity oracle
- **Test plan** (requirementDiagram): 7 `rs/#[test]` elements 1:1 with R1-R3, R5-R8. R8 (JIT no-garbage / no-crash) is the explicit acceptance gate.

## Done

- CRRR through `td_merged + closed` on `issue-bug-tuple-swap-unpack-a-b-b-a-b-produces-garb`.
- Spec authored, reviewed (approved review #1), merged into the issue body and the tech_design tree.
- GitHub #2023 state: CLOSED, phase:td_merged, ship:step1_shipped.

## Next (operator)

1. **FF-merge** `issue-bug-tuple-swap-unpack-a-b-b-a-b-produces-garb` into `project-mamba`; delete the issue branch. Same pattern as #2024.
2. **Hand-write** the implementation per the spec — both the lowering fix (`assign.rs`) and the stack-allocated `RhsTempBuffer` (`unpack.rs`), plus the conformance test + Python fixture.
3. **Run the acceptance gate** — `cargo test -p mamba-conformance tuple_unpack_eval_order` (Fibonacci-step swap, 3-target rotate, length-mismatch ValueError, JIT no-garbage). Today the n=2 swap test should crash the JIT; after the fix, all 7 tests pass on both interpreter and JIT.

## Success criteria

- `a, b = b, a + b` after one iteration leaves `a` = old `b`, `b` = old `a + b`. No garbage. (R1)
- `a, b, c = c, a, b` rotates correctly for n>=3. (R2)
- Length mismatch raises `ValueError: too many / not enough values to unpack`, matching CPython 3.12. (R3)
- Small-N (n<=8) unpack does not heap-allocate. (R5)
- Single-target `t = (b, a + b)` still works (regression guard). (R6)
- 10-iteration Fibonacci-step swap matches CPython 3.12 byte-for-byte. (R7)
- JIT backend no longer produces garbage / crashes on the n=2 swap. (R8 — acceptance gate)

## Notes

- The in-place CRRR workflow kept everything on `issue-*` (no `td-*` ever created), so the cleanup is FF + delete of the issue branch — NOT the State-B `td-*` flow.
- Same known bugs apply as in #2024's handoff: `score td validate <numeric-slug>` looks up by numeric filename (workaround used elsewhere — but the long-slug path worked here, no rename needed); `score td merge` git-merges the whole branch into current (which is fine since patrol stays on the issue branch).
- Per repo memory `feedback_handwritten_temp_then_codegen.md`, the hand-written fix is OK as TEMP state — the slug is "done" only when the lowering/assign.rs change is regenerable from the spec via codegen. That's a follow-up workstream, not this issue's acceptance.
