# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "api_heappushpop_is_present"
# subject = "heapq.heappushpop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""heapq.heappushpop: api_heappushpop_is_present (surface)."""
import heapq

assert hasattr(heapq, "heappushpop")
print("api_heappushpop_is_present OK")
