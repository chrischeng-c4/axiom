# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_pow_is_present"
# subject = "math.pow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.pow: api_pow_is_present (surface)."""
import math

assert hasattr(math, "pow")
print("api_pow_is_present OK")
