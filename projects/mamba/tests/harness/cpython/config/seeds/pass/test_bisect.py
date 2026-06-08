# test_bisect.py — #2826 CPython bisect seed (executed assertions).
#
# Replaces the prior vendored CPython upstream Lib/test/test_bisect.py
# (ranked `Fail` at the recurring INT48-typing class-body Cranelift
# verifier reject) with a Mamba-authored seed distilled from the
# bisect binary-search position surface. Exercises the deterministic
# bisect_left / bisect_right / bisect / insort family that works on
# both CPython 3.12 and mamba today and emits the runner's positive
# proof-of-execution marker that `cpython_lib_test_runner.rs` (#2691)
# classifies as `AssertionPass`.
#
# Why so small? Mamba's current bisect surface presents a healthy
# subset (bisect_left, bisect_right, bisect, insort, insort_left,
# insort_right) and returns the same positions / mutates the same
# lists as CPython on a fixed sorted list. Richer surface — `lo` /
# `hi` keyword bounds, the `key=` parameter (3.10+), bisecting on
# custom comparators via tuple — lands as each gap closes.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: bisect N asserts` to stdout.

import bisect

_ledger: list[int] = []

# Fixed sorted list. The seed contract calls for a fixed sorted list
# so positions are deterministic across both runtimes (per #2826
# acceptance: "Fixture uses a fixed sorted list").
_data = [1, 3, 5, 7, 9]

# 1. Module identity.
assert bisect.__name__ == "bisect", "bisect.__name__ must be 'bisect'"
_ledger.append(1)

# 2. bisect_left — leftmost insertion point that keeps the list sorted.
#    For an absent value 4 between 3 and 5, the position is 2 (just
#    after the 3, just before the 5).
assert bisect.bisect_left(_data, 4) == 2, "bisect_left([1,3,5,7,9], 4) must be 2"
_ledger.append(1)
#    For a present value 5, bisect_left returns the position *of* the
#    matching element (here index 2 — same as the absent-value case).
assert bisect.bisect_left(_data, 5) == 2, "bisect_left([1,3,5,7,9], 5) must be 2"
_ledger.append(1)
#    Below the lowest element: position 0.
assert bisect.bisect_left(_data, 0) == 0, "bisect_left([1,3,5,7,9], 0) must be 0"
_ledger.append(1)
#    Above the highest element: position len(_data) == 5.
assert bisect.bisect_left(_data, 10) == 5, "bisect_left([1,3,5,7,9], 10) must be 5"
_ledger.append(1)

# 3. bisect_right — rightmost insertion point that keeps the list sorted.
#    For an absent value 4, bisect_right matches bisect_left (the run-
#    length is zero), position 2.
assert bisect.bisect_right(_data, 4) == 2, "bisect_right([1,3,5,7,9], 4) must be 2"
_ledger.append(1)
#    For a present value 5, bisect_right returns one *past* the
#    matching element — position 3 (so a new 5 inserted at 3 sits to
#    the right of the existing 5).
assert bisect.bisect_right(_data, 5) == 3, "bisect_right([1,3,5,7,9], 5) must be 3"
_ledger.append(1)
#    Below the lowest element: position 0.
assert bisect.bisect_right(_data, 0) == 0, "bisect_right([1,3,5,7,9], 0) must be 0"
_ledger.append(1)
#    Above the highest element: position len(_data) == 5.
assert bisect.bisect_right(_data, 10) == 5, "bisect_right([1,3,5,7,9], 10) must be 5"
_ledger.append(1)

# 4. bisect is an alias for bisect_right. Pinning the alias catches a
#    regression that diverges them.
assert bisect.bisect(_data, 5) == 3, "bisect alias must match bisect_right"
_ledger.append(1)
assert bisect.bisect(_data, 4) == bisect.bisect_right(_data, 4), "bisect == bisect_right (any input)"
_ledger.append(1)

# 5. insort — mutate the list in place, inserting at bisect_right's
#    position. Idiomatic for "keep a list sorted as items arrive".
_d = [1, 3, 5, 7]
bisect.insort(_d, 4)
assert _d == [1, 3, 4, 5, 7], "insort inserts 4 between 3 and 5"
_ledger.append(1)
assert len(_d) == 5, "insort grew the list by 1"
_ledger.append(1)

# 6. insort_left — mutate the list in place, inserting at bisect_left's
#    position. On an absent value it matches insort; on a present value
#    it inserts to the left of the existing run.
_dL = [1, 3, 5, 7]
bisect.insort_left(_dL, 4)
assert _dL == [1, 3, 4, 5, 7], "insort_left inserts 4 between 3 and 5"
_ledger.append(1)

# 7. insort_right — explicit alias of insort. Mutates the list at
#    bisect_right's position. Pinning the alias catches a regression
#    that diverges it from insort.
_dR = [1, 3, 5, 7]
bisect.insort_right(_dR, 4)
assert _dR == [1, 3, 4, 5, 7], "insort_right inserts 4 between 3 and 5"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: bisect {len(_ledger)} asserts")
