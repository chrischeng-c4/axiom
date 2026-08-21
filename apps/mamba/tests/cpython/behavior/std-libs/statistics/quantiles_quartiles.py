# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "quantiles_quartiles"
# subject = "statistics.quantiles"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.quantiles: quantiles(n=4) returns the three quartiles, with the middle quartile equal to the median over 1..100 (~50.5)"""
import statistics

# quantiles(n=4) gives the three quartiles Q1, Q2, Q3.
_data = list(range(1, 101))  # 1..100
_q = statistics.quantiles(_data, n=4)
assert len(_q) == 3, len(_q)
# The middle quartile is the median, ~50.5 over 1..100.
assert abs(_q[1] - 50.5) < 1, _q[1]
assert _q[1] == statistics.median(_data)

print("quantiles_quartiles OK")
