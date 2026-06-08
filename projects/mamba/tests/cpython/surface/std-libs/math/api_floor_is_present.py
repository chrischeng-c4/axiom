# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_floor_is_present"
# subject = "math.floor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.floor: api_floor_is_present (surface)."""
import math

assert hasattr(math, "floor")
print("api_floor_is_present OK")
