# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "bisect_alias_equals_bisect_right"
# subject = "bisect.bisect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect: bisect is an alias for bisect_right and returns the identical index"""
import bisect

lst = [1, 2, 4, 4, 5, 7]
assert bisect.bisect(lst, 4) == bisect.bisect_right(lst, 4), "bisect == bisect_right"

print("bisect_alias_equals_bisect_right OK")
