# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_asinh_is_present"
# subject = "cmath.asinh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.asinh: api_asinh_is_present (surface)."""
import cmath

assert hasattr(cmath, "asinh")
print("api_asinh_is_present OK")
