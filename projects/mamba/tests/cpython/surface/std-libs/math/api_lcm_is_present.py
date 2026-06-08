# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_lcm_is_present"
# subject = "math.lcm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.lcm: api_lcm_is_present (surface)."""
import math

assert hasattr(math, "lcm")
print("api_lcm_is_present OK")
