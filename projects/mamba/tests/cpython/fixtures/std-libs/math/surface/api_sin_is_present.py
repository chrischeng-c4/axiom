# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_sin_is_present"
# subject = "math.sin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.sin: api_sin_is_present (surface)."""
import math

assert hasattr(math, "sin")
print("api_sin_is_present OK")
