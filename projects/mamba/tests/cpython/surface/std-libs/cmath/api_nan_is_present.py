# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_nan_is_present"
# subject = "cmath.nan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.nan: api_nan_is_present (surface)."""
import cmath

assert hasattr(cmath, "nan")
print("api_nan_is_present OK")
