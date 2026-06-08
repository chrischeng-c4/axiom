# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "harmonic_mean_contract"
# subject = "statistics.harmonic_mean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.harmonic_mean: harmonic_mean is the reciprocal of the mean of reciprocals ([1,2,4] -> 1.7142857142857142), a single zero collapses it to 0, a singleton is the value itself, and weights repeat data points (harmonic_mean([40,60],[5,30]) == 56.0)"""
from statistics import harmonic_mean

# Reciprocal of the mean of reciprocals.
assert harmonic_mean([1.0, 2.0, 4.0]) == 1.7142857142857142
# A single zero collapses the harmonic mean to 0.
assert harmonic_mean([1, 0, 2]) == 0
# A singleton harmonic mean is the value itself.
assert harmonic_mean([5]) == 5
# Weights repeat data points (exact Fraction arithmetic), positional or keyword.
assert harmonic_mean([40, 60], [5, 30]) == 56.0
assert harmonic_mean([40, 60], weights=[5, 30]) == 56.0

print("harmonic_mean_contract OK")
