# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_e_is_present"
# subject = "cmath.e"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.e: api_e_is_present (surface)."""
import cmath

assert hasattr(cmath, "e")
print("api_e_is_present OK")
