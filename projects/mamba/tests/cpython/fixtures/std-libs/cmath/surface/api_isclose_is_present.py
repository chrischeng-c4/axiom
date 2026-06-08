# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_isclose_is_present"
# subject = "cmath.isclose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.isclose: api_isclose_is_present (surface)."""
import cmath

assert hasattr(cmath, "isclose")
print("api_isclose_is_present OK")
