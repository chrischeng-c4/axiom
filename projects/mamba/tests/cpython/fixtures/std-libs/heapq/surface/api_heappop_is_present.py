# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "api_heappop_is_present"
# subject = "heapq.heappop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""heapq.heappop: api_heappop_is_present (surface)."""
import heapq

assert hasattr(heapq, "heappop")
print("api_heappop_is_present OK")
