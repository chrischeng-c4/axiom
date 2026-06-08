# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_tan_is_present"
# subject = "cmath.tan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.tan: api_tan_is_present (surface)."""
import cmath

assert hasattr(cmath, "tan")
print("api_tan_is_present OK")
