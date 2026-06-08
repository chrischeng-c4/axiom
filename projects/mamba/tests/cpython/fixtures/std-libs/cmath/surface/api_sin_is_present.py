# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_sin_is_present"
# subject = "cmath.sin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.sin: api_sin_is_present (surface)."""
import cmath

assert hasattr(cmath, "sin")
print("api_sin_is_present OK")
