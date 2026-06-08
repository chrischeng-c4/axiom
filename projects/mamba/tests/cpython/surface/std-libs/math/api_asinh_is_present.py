# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_asinh_is_present"
# subject = "math.asinh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.asinh: api_asinh_is_present (surface)."""
import math

assert hasattr(math, "asinh")
print("api_asinh_is_present OK")
