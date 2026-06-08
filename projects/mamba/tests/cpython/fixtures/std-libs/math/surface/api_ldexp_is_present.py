# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_ldexp_is_present"
# subject = "math.ldexp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.ldexp: api_ldexp_is_present (surface)."""
import math

assert hasattr(math, "ldexp")
print("api_ldexp_is_present OK")
