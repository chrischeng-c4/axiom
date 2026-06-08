# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_nan_is_present"
# subject = "math.nan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.nan: api_nan_is_present (surface)."""
import math

assert hasattr(math, "nan")
print("api_nan_is_present OK")
