# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_isnan_is_present"
# subject = "math.isnan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.isnan: api_isnan_is_present (surface)."""
import math

assert hasattr(math, "isnan")
print("api_isnan_is_present OK")
