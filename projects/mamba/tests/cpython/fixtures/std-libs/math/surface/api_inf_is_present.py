# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_inf_is_present"
# subject = "math.inf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.inf: api_inf_is_present (surface)."""
import math

assert hasattr(math, "inf")
print("api_inf_is_present OK")
