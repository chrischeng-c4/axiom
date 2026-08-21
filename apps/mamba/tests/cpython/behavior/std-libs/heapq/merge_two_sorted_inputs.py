# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_two_sorted_inputs"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() lazily combines two already-sorted inputs into one ascending iterator"""
import heapq

_m = list(heapq.merge([1, 3, 5], [2, 4, 6]))
assert _m == [1, 2, 3, 4, 5, 6], f"merge = {_m!r}"
print("merge_two_sorted_inputs OK")
