# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "api_bisect_right_is_present"
# subject = "bisect.bisect_right"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""bisect.bisect_right: api_bisect_right_is_present (surface)."""
import bisect

assert hasattr(bisect, "bisect_right")
print("api_bisect_right_is_present OK")
