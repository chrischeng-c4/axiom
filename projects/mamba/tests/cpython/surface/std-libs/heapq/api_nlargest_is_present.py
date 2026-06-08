# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "heapq"
# dimension = "surface"
# case = "api_nlargest_is_present"
# subject = "heapq.nlargest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""heapq.nlargest: api_nlargest_is_present (surface)."""
import heapq

assert hasattr(heapq, "nlargest")
print("api_nlargest_is_present OK")
