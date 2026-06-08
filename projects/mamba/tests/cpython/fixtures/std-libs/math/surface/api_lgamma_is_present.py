# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_lgamma_is_present"
# subject = "math.lgamma"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.lgamma: api_lgamma_is_present (surface)."""
import math

assert hasattr(math, "lgamma")
print("api_lgamma_is_present OK")
