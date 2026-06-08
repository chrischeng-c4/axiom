# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_log10_is_present"
# subject = "cmath.log10"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.log10: api_log10_is_present (surface)."""
import cmath

assert hasattr(cmath, "log10")
print("api_log10_is_present OK")
