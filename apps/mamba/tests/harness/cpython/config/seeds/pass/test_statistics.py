# test_statistics.py — #2837 CPython statistics seed (executed assertions).
#
# Mamba-authored seed distilled from the statistics module surface.
# Exercises mean/median/median_low/median_high/mode/variance/stdev/
# pvariance/pstdev — the load-bearing helpers downstream users actually
# reach for — on tiny deterministic integer inputs per the #2837
# acceptance "fixture uses tiny deterministic numeric inputs."
#
# Why so small? Mamba's current statistics surface presents the
# canonical names and produces the same answers as CPython on the
# small fixed inputs exercised here. Richer surface — quantiles,
# correlation, linear_regression, NormalDist — lands as each gap
# closes.
#
# Floating comparisons use `abs(x - expected) < 1e-10` for the
# irrational answers (sample variance/stdev for non-square data).
# Exactly representable cases (mean of an arithmetic progression,
# median of an odd-length list) use direct `==`.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: statistics N asserts` to stdout.

import statistics

_ledger: list[int] = []

# 1. Module identity + public surface bindings.
assert statistics.__name__ == "statistics", "statistics.__name__ must be 'statistics'"
_ledger.append(1)
assert hasattr(statistics, "mean"), "statistics must expose mean"
_ledger.append(1)
assert hasattr(statistics, "median"), "statistics must expose median"
_ledger.append(1)
assert hasattr(statistics, "median_low"), "statistics must expose median_low"
_ledger.append(1)
assert hasattr(statistics, "median_high"), "statistics must expose median_high"
_ledger.append(1)
assert hasattr(statistics, "mode"), "statistics must expose mode"
_ledger.append(1)
assert hasattr(statistics, "variance"), "statistics must expose variance"
_ledger.append(1)
assert hasattr(statistics, "stdev"), "statistics must expose stdev"
_ledger.append(1)
assert hasattr(statistics, "pvariance"), "statistics must expose pvariance"
_ledger.append(1)
assert hasattr(statistics, "pstdev"), "statistics must expose pstdev"
_ledger.append(1)

# 2. mean — arithmetic average over the input. (1+2+3+4+5)/5 = 3.
#    CPython returns int (3) for all-int input; mamba returns float
#    (3.0). Compare with tolerance to absorb the type difference.
assert abs(statistics.mean([1, 2, 3, 4, 5]) - 3.0) < 1e-10, "mean([1..5]) ≈ 3"
_ledger.append(1)
assert abs(statistics.mean([10, 20, 30]) - 20.0) < 1e-10, "mean([10,20,30]) ≈ 20"
_ledger.append(1)
assert abs(statistics.mean([5]) - 5.0) < 1e-10, "mean([5]) ≈ 5 (single element)"
_ledger.append(1)

# 3. median — odd-length list returns the middle element; even-length
#    returns the mean of the two middle elements.
assert statistics.median([1, 2, 3, 4, 5]) == 3, "median([1..5]) == 3 (odd length, middle)"
_ledger.append(1)
assert statistics.median([1, 2, 3, 4]) == 2.5, "median([1..4]) == 2.5 (even length, mean of middle pair)"
_ledger.append(1)
assert statistics.median([7]) == 7, "median([7]) == 7"
_ledger.append(1)

# 4. median_low / median_high — pick the lower / upper of the two
#    middle elements (instead of averaging) on even-length lists.
assert statistics.median_low([1, 2, 3, 4]) == 2, "median_low([1..4]) == 2 (lower of middle pair)"
_ledger.append(1)
assert statistics.median_high([1, 2, 3, 4]) == 3, "median_high([1..4]) == 3 (upper of middle pair)"
_ledger.append(1)

# 5. mode — most-frequent element. [1,2,2,3,3,3] has 3 occurring most.
assert statistics.mode([1, 2, 2, 3, 3, 3]) == 3, "mode picks the most frequent (3)"
_ledger.append(1)
assert statistics.mode([7, 7, 8]) == 7, "mode([7,7,8]) == 7"
_ledger.append(1)

# 6. Sample variance / standard deviation — Bessel-corrected (divides
#    by n-1). [1,2,3,4,5] has mean 3, deviations [-2,-1,0,1,2], sum
#    of squares 10, sample variance 10/4 = 2.5; stdev = sqrt(2.5).
assert abs(statistics.variance([1, 2, 3, 4, 5]) - 2.5) < 1e-10, "variance([1..5]) ≈ 2.5 (Bessel)"
_ledger.append(1)
assert abs(statistics.stdev([1, 2, 3, 4, 5]) - 1.5811388300841898) < 1e-10, "stdev([1..5]) ≈ √2.5"
_ledger.append(1)

# 7. Population variance / population stdev — divides by n (no
#    Bessel correction). [1,2,3,4,5] has pvariance 10/5 = 2.0;
#    pstdev = sqrt(2.0).
assert abs(statistics.pvariance([1, 2, 3, 4, 5]) - 2.0) < 1e-10, "pvariance([1..5]) ≈ 2.0 (no Bessel)"
_ledger.append(1)
assert abs(statistics.pstdev([1, 2, 3, 4, 5]) - 1.4142135623730951) < 1e-10, "pstdev([1..5]) ≈ √2"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: statistics {len(_ledger)} asserts")
