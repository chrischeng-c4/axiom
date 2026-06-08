# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "absent_element_insertion_point"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: for an absent element both bisect_left and bisect_right give the same insertion index"""
import bisect

b = [10, 20, 30, 40]
assert bisect.bisect_left(b, 25) == 2, f"absent elem left = {bisect.bisect_left(b, 25)!r}"
assert bisect.bisect_right(b, 25) == 2, f"absent elem right = {bisect.bisect_right(b, 25)!r}"

print("absent_element_insertion_point OK")
