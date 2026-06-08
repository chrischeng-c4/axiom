# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_remainder_is_present"
# subject = "math.remainder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.remainder: api_remainder_is_present (surface)."""
import math

assert hasattr(math, "remainder")
print("api_remainder_is_present OK")
