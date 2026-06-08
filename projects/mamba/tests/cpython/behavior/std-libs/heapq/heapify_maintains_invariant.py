# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "behavior"
# case = "heapify_maintains_invariant"
# subject = "heapq.heapify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.heapify: after heapify every parent heap[i] is <= its children heap[2i+1] and heap[2i+2] (the min-heap invariant)"""
import heapq

_lst = [9, 4, 7, 2, 5]
heapq.heapify(_lst)
assert _lst[0] == 2, f"heapify min = {_lst[0]!r}"
# Heap invariant: for all i, heap[i] <= heap[2*i+1] and heap[2*i+2].
for _i in range(len(_lst)):
    if 2 * _i + 1 < len(_lst):
        assert _lst[_i] <= _lst[2 * _i + 1], f"heap invariant left child at {_i}"
    if 2 * _i + 2 < len(_lst):
        assert _lst[_i] <= _lst[2 * _i + 2], f"heap invariant right child at {_i}"
print("heapify_maintains_invariant OK")
