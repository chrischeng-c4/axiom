# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_frexp_is_present"
# subject = "math.frexp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.frexp: api_frexp_is_present (surface)."""
import math

assert hasattr(math, "frexp")
print("api_frexp_is_present OK")
