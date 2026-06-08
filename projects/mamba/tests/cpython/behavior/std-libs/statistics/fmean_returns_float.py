# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "fmean_returns_float"
# subject = "statistics.fmean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.fmean: fmean always returns a float regardless of input numeric type (float/Decimal/Fraction/bool, tuple, iterator) and its weights act as repetition counts so uniform weights equal the unweighted mean"""
from decimal import Decimal
from fractions import Fraction
from statistics import fmean

# fmean always returns a float, whatever the input numeric type or container.
for data, expected in [([3.5, 4.0, 5.25], 4.25),
                       ([Decimal("3.5"), Decimal("4.0"), Decimal("5.25")], 4.25),
                       ([Fraction(7, 2), Fraction(4, 1), Fraction(21, 4)], 4.25),
                       ([True, False, True, True, False], 0.6),
                       ((3.5, 4.0, 5.25), 4.25),
                       (iter([3.5, 4.0, 5.25]), 4.25)]:
    got = fmean(data)
    assert type(got) is float and got == expected, (got, expected)
# Weights act as repetition counts; uniform weights == the unweighted mean.
assert fmean([10, 10, 20], [0.25, 0.25, 0.5]) == fmean([10, 10, 20, 20])

print("fmean_returns_float OK")
