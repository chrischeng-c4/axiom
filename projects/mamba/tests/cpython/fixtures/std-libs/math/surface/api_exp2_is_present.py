# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_exp2_is_present"
# subject = "math.exp2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.exp2: api_exp2_is_present (surface)."""
import math

assert hasattr(math, "exp2")
print("api_exp2_is_present OK")
