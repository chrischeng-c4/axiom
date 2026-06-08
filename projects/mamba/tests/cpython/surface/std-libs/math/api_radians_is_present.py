# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_radians_is_present"
# subject = "math.radians"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.radians: api_radians_is_present (surface)."""
import math

assert hasattr(math, "radians")
print("api_radians_is_present OK")
