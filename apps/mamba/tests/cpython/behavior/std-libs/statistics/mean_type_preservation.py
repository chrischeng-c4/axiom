# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "mean_type_preservation"
# subject = "statistics.mean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mean: mean of an all-int list reduces to the exact value (3 for 1..5, 2.5 for 1..4) and mean of floats returns a float"""
import statistics

# An all-int list whose average is a whole number reduces to that value.
assert statistics.mean([1, 2, 3, 4, 5]) == 3, statistics.mean([1, 2, 3, 4, 5])
# An all-int list whose average is fractional yields the exact 2.5.
assert statistics.mean([1, 2, 3, 4]) == 2.5, statistics.mean([1, 2, 3, 4])
# Mean of floats is a float.
_mf = statistics.mean([1.0, 2.0, 3.0])
assert _mf == 2.0, _mf
assert statistics.mean([1.5, 2.5]) == 2.0, statistics.mean([1.5, 2.5])

print("mean_type_preservation OK")
