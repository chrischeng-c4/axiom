# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_copysign_is_present"
# subject = "math.copysign"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.copysign: api_copysign_is_present (surface)."""
import math

assert hasattr(math, "copysign")
print("api_copysign_is_present OK")
