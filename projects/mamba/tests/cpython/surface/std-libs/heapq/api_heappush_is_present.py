# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "api_heappush_is_present"
# subject = "heapq.heappush"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""heapq.heappush: api_heappush_is_present (surface)."""
import heapq

assert hasattr(heapq, "heappush")
print("api_heappush_is_present OK")
