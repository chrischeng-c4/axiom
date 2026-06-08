# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "merge_is_callable"
# subject = "heapq.merge"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""heapq.merge: merge_is_callable (surface)."""
import heapq

assert callable(heapq.merge)
print("merge_is_callable OK")
