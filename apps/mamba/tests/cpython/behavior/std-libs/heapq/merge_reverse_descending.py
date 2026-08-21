# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_reverse_descending"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge(reverse=True) merges descending inputs into a single descending result"""
import heapq

desc_a = [9, 6, 3]
desc_b = [8, 5, 2]
merged_desc = list(heapq.merge(desc_a, desc_b, reverse=True))
assert merged_desc == [9, 8, 6, 5, 3, 2], f"merge reverse= = {merged_desc!r}"
print("merge_reverse_descending OK")
