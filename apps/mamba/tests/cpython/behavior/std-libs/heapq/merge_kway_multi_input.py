# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_kway_multi_input"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge() combines more than two sorted inputs k-way in a single pass"""
import heapq

multi = list(heapq.merge([1, 5], [2, 6], [3, 7], [4, 8]))
assert multi == [1, 2, 3, 4, 5, 6, 7, 8], f"k-way merge = {multi!r}"
print("merge_kway_multi_input OK")
