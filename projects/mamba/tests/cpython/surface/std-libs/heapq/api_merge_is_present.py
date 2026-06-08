# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "api_merge_is_present"
# subject = "heapq.merge"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""heapq.merge: api_merge_is_present (surface)."""
import heapq

assert hasattr(heapq, "merge")
print("api_merge_is_present OK")
