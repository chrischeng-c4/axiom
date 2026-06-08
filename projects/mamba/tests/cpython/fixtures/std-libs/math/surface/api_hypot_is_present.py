# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_hypot_is_present"
# subject = "math.hypot"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.hypot: api_hypot_is_present (surface)."""
import math

assert hasattr(math, "hypot")
print("api_hypot_is_present OK")
