# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_trunc_is_present"
# subject = "math.trunc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.trunc: api_trunc_is_present (surface)."""
import math

assert hasattr(math, "trunc")
print("api_trunc_is_present OK")
