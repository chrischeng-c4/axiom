# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_dist_is_present"
# subject = "math.dist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.dist: api_dist_is_present (surface)."""
import math

assert hasattr(math, "dist")
print("api_dist_is_present OK")
