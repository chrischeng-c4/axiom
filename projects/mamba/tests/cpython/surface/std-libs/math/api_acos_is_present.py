# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_acos_is_present"
# subject = "math.acos"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.acos: api_acos_is_present (surface)."""
import math

assert hasattr(math, "acos")
print("api_acos_is_present OK")
