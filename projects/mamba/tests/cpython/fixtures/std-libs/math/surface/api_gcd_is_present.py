# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_gcd_is_present"
# subject = "math.gcd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.gcd: api_gcd_is_present (surface)."""
import math

assert hasattr(math, "gcd")
print("api_gcd_is_present OK")
