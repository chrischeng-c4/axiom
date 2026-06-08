# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_atanh_is_present"
# subject = "math.atanh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.atanh: api_atanh_is_present (surface)."""
import math

assert hasattr(math, "atanh")
print("api_atanh_is_present OK")
