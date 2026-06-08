# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_exp_is_present"
# subject = "math.exp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.exp: api_exp_is_present (surface)."""
import math

assert hasattr(math, "exp")
print("api_exp_is_present OK")
