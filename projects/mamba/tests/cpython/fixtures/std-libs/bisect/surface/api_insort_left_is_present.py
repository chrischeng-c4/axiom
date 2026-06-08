# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "api_insort_left_is_present"
# subject = "bisect.insort_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""bisect.insort_left: api_insort_left_is_present (surface)."""
import bisect

assert hasattr(bisect, "insort_left")
print("api_insort_left_is_present OK")
