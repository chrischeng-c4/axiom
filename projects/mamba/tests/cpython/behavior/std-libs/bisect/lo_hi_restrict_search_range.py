# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "lo_hi_restrict_search_range"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: lo and hi confine the binary search to the [lo, hi) sub-window"""
import bisect

e = [0, 1, 2, 3, 4, 5]
# search restricted to [2, 5): element 3 maps to index 3
assert bisect.bisect_left(e, 3, lo=2, hi=5) == 3, f"lo/hi range = {bisect.bisect_left(e, 3, lo=2, hi=5)!r}"
# a value below the window clamps to lo
assert bisect.bisect_left(e, 1, lo=2, hi=4) == 2, "below-window clamps to lo"
# a value above the window clamps to hi
assert bisect.bisect_left(e, 9, lo=2, hi=4) == 4, "above-window clamps to hi"

print("lo_hi_restrict_search_range OK")
