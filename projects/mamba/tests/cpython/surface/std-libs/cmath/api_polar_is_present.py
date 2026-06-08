# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_polar_is_present"
# subject = "cmath.polar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.polar: api_polar_is_present (surface)."""
import cmath

assert hasattr(cmath, "polar")
print("api_polar_is_present OK")
