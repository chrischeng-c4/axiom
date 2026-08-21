# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heapify_min_at_root"
# subject = "heapq.heapify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: heapify rearranges a list in place so the minimum element sits at index 0 (the root)"""
import heapq

_lst = [10, 5, 3, 8, 1]
heapq.heapify(_lst)
assert _lst[0] == min([10, 5, 3, 8, 1]), f"heapify min at root = {_lst[0]!r}"
print("heapify_min_at_root OK")
