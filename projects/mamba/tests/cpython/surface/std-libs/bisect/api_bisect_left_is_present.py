# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "api_bisect_left_is_present"
# subject = "bisect.bisect_left"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""bisect.bisect_left: api_bisect_left_is_present (surface)."""
import bisect

assert hasattr(bisect, "bisect_left")
print("api_bisect_left_is_present OK")
