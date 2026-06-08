# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "api_heapify_is_present"
# subject = "heapq.heapify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""heapq.heapify: api_heapify_is_present (surface)."""
import heapq

assert hasattr(heapq, "heapify")
print("api_heapify_is_present OK")
