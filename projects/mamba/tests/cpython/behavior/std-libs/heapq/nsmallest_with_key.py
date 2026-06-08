# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "nsmallest_with_key"
# subject = "heapq.nsmallest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.nsmallest: nsmallest(n, data, key=...) selects the n elements with the smallest projected key, smallest first"""
import heapq

_nums = [(-3, "a"), (1, "b"), (-1, "c"), (2, "d")]
_small2 = heapq.nsmallest(2, _nums, key=lambda x: x[0])
assert _small2[0][0] == -3, f"nsmallest by key = {_small2!r}"
print("nsmallest_with_key OK")
