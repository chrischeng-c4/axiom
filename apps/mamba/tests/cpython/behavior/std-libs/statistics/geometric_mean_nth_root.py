# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "geometric_mean_nth_root"
# subject = "statistics.geometric_mean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.geometric_mean: geometric_mean is the nth root of the product (geometric_mean([54,24,36]) ~ 36.0, [4.0,9.0] ~ 6.0)"""
from statistics import geometric_mean

# geometric_mean is the nth root of the product of the data.
assert abs(geometric_mean([54, 24, 36]) - 36.0) < 1e-7, geometric_mean([54, 24, 36])
assert abs(geometric_mean([4.0, 9.0]) - 6.0) < 1e-7, geometric_mean([4.0, 9.0])
assert abs(geometric_mean([1, 2, 4, 8]) - 2.8284271247461903) < 1e-9

print("geometric_mean_nth_root OK")
