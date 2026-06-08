# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_isfinite_is_present"
# subject = "cmath.isfinite"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.isfinite: api_isfinite_is_present (surface)."""
import cmath

assert hasattr(cmath, "isfinite")
print("api_isfinite_is_present OK")
