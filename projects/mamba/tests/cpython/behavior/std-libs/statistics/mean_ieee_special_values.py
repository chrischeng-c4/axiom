# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "mean_ieee_special_values"
# subject = "statistics.mean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mean: a single nan poisons the mean (float and Decimal alike), a single signed infinity dominates and is preserved, and +inf with -inf yields nan rather than an error"""
import math
import statistics
from decimal import Decimal

# A single nan poisons the mean, for both float and Decimal inputs.
for kind in (float, Decimal):
    _r = statistics.mean([1, 3, 5, 7, 9, kind("nan")])
    assert math.isnan(_r), repr(_r)
# A single infinity dominates and is preserved with its sign.
for kind in (float, Decimal):
    for sign in (1, -1):
        _inf = kind("inf") * sign
        _r = statistics.mean([1, 3, 5, 7, 9, _inf])
        assert math.isinf(_r) and _r == _inf, repr(_r)
# +inf and -inf together yield nan, not an error.
assert math.isnan(statistics.mean([2, 4, 6, float("inf"), 1, float("-inf")]))

print("mean_ieee_special_values OK")
