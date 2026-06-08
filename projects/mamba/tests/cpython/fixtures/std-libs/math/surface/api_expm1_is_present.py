# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_expm1_is_present"
# subject = "math.expm1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.expm1: api_expm1_is_present (surface)."""
import math

assert hasattr(math, "expm1")
print("api_expm1_is_present OK")
