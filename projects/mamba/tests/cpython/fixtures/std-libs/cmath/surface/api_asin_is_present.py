# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_asin_is_present"
# subject = "cmath.asin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.asin: api_asin_is_present (surface)."""
import cmath

assert hasattr(cmath, "asin")
print("api_asin_is_present OK")
