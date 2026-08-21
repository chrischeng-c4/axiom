# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "empty_list_returns_zero"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect_left and bisect_right on an empty list both return 0"""
import bisect

assert bisect.bisect_left([], 5) == 0, "empty list bisect_left = 0"
assert bisect.bisect_right([], 5) == 0, "empty list bisect_right = 0"

print("empty_list_returns_zero OK")
