# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "api_bisect_is_present"
# subject = "bisect.bisect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""bisect.bisect: api_bisect_is_present (surface)."""
import bisect

assert hasattr(bisect, "bisect")
print("api_bisect_is_present OK")
