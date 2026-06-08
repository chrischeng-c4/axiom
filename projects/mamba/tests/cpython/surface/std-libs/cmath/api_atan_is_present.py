# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_atan_is_present"
# subject = "cmath.atan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.atan: api_atan_is_present (surface)."""
import cmath

assert hasattr(cmath, "atan")
print("api_atan_is_present OK")
