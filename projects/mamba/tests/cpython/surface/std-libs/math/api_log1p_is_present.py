# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_log1p_is_present"
# subject = "math.log1p"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.log1p: api_log1p_is_present (surface)."""
import math

assert hasattr(math, "log1p")
print("api_log1p_is_present OK")
