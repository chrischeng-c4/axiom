# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_comb_is_present"
# subject = "math.comb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.comb: api_comb_is_present (surface)."""
import math

assert hasattr(math, "comb")
print("api_comb_is_present OK")
