# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_degrees_is_present"
# subject = "math.degrees"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.degrees: api_degrees_is_present (surface)."""
import math

assert hasattr(math, "degrees")
print("api_degrees_is_present OK")
