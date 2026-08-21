# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "bisect_left_leftmost_of_equal"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect_left returns the insertion point left of all equal elements"""
import bisect

a = [1, 2, 2, 2, 5]
assert bisect.bisect_left(a, 2) == 1, f"bisect_left leftmost = {bisect.bisect_left(a, 2)!r}"

print("bisect_left_leftmost_of_equal OK")
