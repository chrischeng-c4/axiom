# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_isnan_is_present"
# subject = "cmath.isnan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.isnan: api_isnan_is_present (surface)."""
import cmath

assert hasattr(cmath, "isnan")
print("api_isnan_is_present OK")
