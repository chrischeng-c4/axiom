# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_cosh_is_present"
# subject = "math.cosh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.cosh: api_cosh_is_present (surface)."""
import math

assert hasattr(math, "cosh")
print("api_cosh_is_present OK")
