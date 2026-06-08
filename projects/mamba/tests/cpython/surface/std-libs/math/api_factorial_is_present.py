# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_factorial_is_present"
# subject = "math.factorial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.factorial: api_factorial_is_present (surface)."""
import math

assert hasattr(math, "factorial")
print("api_factorial_is_present OK")
