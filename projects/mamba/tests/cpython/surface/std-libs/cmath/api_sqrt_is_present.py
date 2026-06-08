# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_sqrt_is_present"
# subject = "cmath.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.sqrt: api_sqrt_is_present (surface)."""
import cmath

assert hasattr(cmath, "sqrt")
print("api_sqrt_is_present OK")
