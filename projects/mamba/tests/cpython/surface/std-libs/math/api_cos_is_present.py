# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_cos_is_present"
# subject = "math.cos"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.cos: api_cos_is_present (surface)."""
import math

assert hasattr(math, "cos")
print("api_cos_is_present OK")
