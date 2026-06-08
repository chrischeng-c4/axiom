# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_atan_is_present"
# subject = "math.atan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.atan: api_atan_is_present (surface)."""
import math

assert hasattr(math, "atan")
print("api_atan_is_present OK")
