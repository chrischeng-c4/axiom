# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "quantiles_exclusive_vs_inclusive"
# subject = "statistics.quantiles"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.quantiles: n cut points produce n-1 boundaries (n=1 -> []), the exclusive default and the 'inclusive' method give the documented CPython values, and the result element type follows the input numeric type"""
from decimal import Decimal
from fractions import Fraction
from statistics import quantiles, median

# n cut points produce n-1 boundaries (exclusive, the default); n=1 -> [].
data = [120, 200, 250, 320, 350]
for n, expected in [(1, []),
                    (2, [250.0]),
                    (4, [160.0, 250.0, 335.0]),
                    (5, [136.0, 220.0, 292.0, 344.0])]:
    got = quantiles(data, n=n)
    assert got == expected, (n, got)
    assert len(got) == n - 1
# The middle quartile equals the median.
q1, q2, q3 = quantiles(data)
assert q2 == median(data)

# The 'inclusive' method uses the data bounds as the extremes.
inc = [100, 200, 400, 800]
assert quantiles(inc, n=4, method="inclusive") == [175.0, 300.0, 500.0]
assert quantiles([0, 100], n=10, method="inclusive") == [
    10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]
# The result element type follows the input numeric type.
for kind in (float, Decimal, Fraction):
    res = quantiles(list(map(kind, inc)), n=4, method="inclusive")
    assert res == list(map(kind, [175, 300, 500])), kind

print("quantiles_exclusive_vs_inclusive OK")
