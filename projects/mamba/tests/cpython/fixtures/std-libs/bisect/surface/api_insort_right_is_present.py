# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "api_insort_right_is_present"
# subject = "bisect.insort_right"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""bisect.insort_right: api_insort_right_is_present (surface)."""
import bisect

assert hasattr(bisect, "insort_right")
print("api_insort_right_is_present OK")
