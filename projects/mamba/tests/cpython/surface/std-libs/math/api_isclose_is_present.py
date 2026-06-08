# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_isclose_is_present"
# subject = "math.isclose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.isclose: api_isclose_is_present (surface)."""
import math

assert hasattr(math, "isclose")
print("api_isclose_is_present OK")
