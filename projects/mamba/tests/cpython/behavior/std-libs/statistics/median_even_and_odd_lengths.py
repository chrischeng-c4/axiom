# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "median_even_and_odd_lengths"
# subject = "statistics.median"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.median: median averages the middle two on even-length input ([1,3,5,7] -> 4) and returns the exact middle on odd-length ([1,3,5] -> 3), keeping int return for odd int input"""
import statistics

# Even-length: average of the two middle values.
assert statistics.median([1, 3, 5, 7]) == 4, statistics.median([1, 3, 5, 7])
assert statistics.median([1, 2, 3, 4]) == 2.5, statistics.median([1, 2, 3, 4])
# Odd-length int input returns the exact middle int.
assert statistics.median([1, 3, 5]) == 3, statistics.median([1, 3, 5])
assert statistics.median([1, 2, 3]) == 2, statistics.median([1, 2, 3])
# Float middle for a float odd-length list.
assert statistics.median([1.5, 2.5, 3.5]) == 2.5, statistics.median([1.5, 2.5, 3.5])

print("median_even_and_odd_lengths OK")
