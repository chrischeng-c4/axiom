# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_erf_is_present"
# subject = "math.erf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.erf: api_erf_is_present (surface)."""
import math

assert hasattr(math, "erf")
print("api_erf_is_present OK")
