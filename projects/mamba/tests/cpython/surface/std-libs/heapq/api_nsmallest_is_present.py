# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "api_nsmallest_is_present"
# subject = "heapq.nsmallest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""heapq.nsmallest: api_nsmallest_is_present (surface)."""
import heapq

assert hasattr(heapq, "nsmallest")
print("api_nsmallest_is_present OK")
