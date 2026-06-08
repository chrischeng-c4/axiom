# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_log10_is_present"
# subject = "math.log10"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.log10: api_log10_is_present (surface)."""
import math

assert hasattr(math, "log10")
print("api_log10_is_present OK")
