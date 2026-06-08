# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "sample_vs_population_stdev"
# subject = "statistics.stdev"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.stdev: sample stdev (ddof=1) exceeds population pstdev (ddof=0) on the same data, with exact CPython values for [2,4,4,4,5,5,7,9]"""
import statistics

_data = [2, 4, 4, 4, 5, 5, 7, 9]
_s = statistics.stdev(_data)   # sample stdev (ddof=1)
_p = statistics.pstdev(_data)  # population stdev (ddof=0)
# Dividing by n-1 (sample) yields a larger value than dividing by n.
assert _s > _p, (_s, _p)
assert abs(_s - 2.138089935299395) < 1e-10, _s
assert abs(_p - 2.0) < 1e-10, _p

print("sample_vs_population_stdev OK")
