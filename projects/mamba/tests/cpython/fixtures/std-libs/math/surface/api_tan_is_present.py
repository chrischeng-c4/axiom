# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_tan_is_present"
# subject = "math.tan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.tan: api_tan_is_present (surface)."""
import math

assert hasattr(math, "tan")
print("api_tan_is_present OK")
