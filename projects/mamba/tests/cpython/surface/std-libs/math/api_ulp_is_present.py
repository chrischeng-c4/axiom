# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_ulp_is_present"
# subject = "math.ulp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.ulp: api_ulp_is_present (surface)."""
import math

assert hasattr(math, "ulp")
print("api_ulp_is_present OK")
