# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "merge_key_projects_order"
# subject = "heapq.merge"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge(key=) merges on the projected key order, not the raw element order, interleaving two row streams by their numeric field"""
import heapq

rows_a = [("A", 1), ("B", 4), ("C", 7)]
rows_b = [("D", 2), ("E", 5), ("F", 8)]
merged_by_num = list(heapq.merge(rows_a, rows_b, key=lambda r: r[1]))
assert merged_by_num == [
    ("A", 1), ("D", 2), ("B", 4), ("E", 5), ("C", 7), ("F", 8)
], f"merge key= = {merged_by_num!r}"
print("merge_key_projects_order OK")
