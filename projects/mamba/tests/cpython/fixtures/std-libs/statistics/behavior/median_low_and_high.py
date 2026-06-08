# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "median_low_and_high"
# subject = "statistics.median_low"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.median_low: median_low picks the lower of the two middle values and median_high the upper on even-length data ([1,3,5,7] -> 3 and 5)"""
import statistics

# On even-length data the two middle values are 3 and 5.
assert statistics.median_low([1, 3, 5, 7]) == 3, statistics.median_low([1, 3, 5, 7])
assert statistics.median_high([1, 3, 5, 7]) == 5, statistics.median_high([1, 3, 5, 7])
# On odd-length data both collapse to the exact middle.
assert statistics.median_low([1, 3, 5]) == 3, statistics.median_low([1, 3, 5])
assert statistics.median_high([1, 3, 5]) == 3, statistics.median_high([1, 3, 5])

print("median_low_and_high OK")
