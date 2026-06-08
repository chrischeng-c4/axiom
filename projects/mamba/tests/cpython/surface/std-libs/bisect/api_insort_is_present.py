# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "api_insort_is_present"
# subject = "bisect.insort"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""bisect.insort: api_insort_is_present (surface)."""
import bisect

assert hasattr(bisect, "insort")
print("api_insort_is_present OK")
