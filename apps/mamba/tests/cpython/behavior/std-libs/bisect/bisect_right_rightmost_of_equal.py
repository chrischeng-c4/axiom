# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "bisect_right_rightmost_of_equal"
# subject = "bisect.bisect_right"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_right: bisect_right returns the insertion point right of all equal elements"""
import bisect

a = [1, 2, 2, 2, 5]
assert bisect.bisect_right(a, 2) == 4, f"bisect_right rightmost = {bisect.bisect_right(a, 2)!r}"

print("bisect_right_rightmost_of_equal OK")
