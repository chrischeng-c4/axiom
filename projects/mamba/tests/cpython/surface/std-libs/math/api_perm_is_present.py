# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_perm_is_present"
# subject = "math.perm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.perm: api_perm_is_present (surface)."""
import math

assert hasattr(math, "perm")
print("api_perm_is_present OK")
