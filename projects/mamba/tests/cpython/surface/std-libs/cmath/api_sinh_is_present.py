# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_sinh_is_present"
# subject = "cmath.sinh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.sinh: api_sinh_is_present (surface)."""
import cmath

assert hasattr(cmath, "sinh")
print("api_sinh_is_present OK")
