# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_isqrt_is_present"
# subject = "math.isqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.isqrt: api_isqrt_is_present (surface)."""
import math

assert hasattr(math, "isqrt")
print("api_isqrt_is_present OK")
