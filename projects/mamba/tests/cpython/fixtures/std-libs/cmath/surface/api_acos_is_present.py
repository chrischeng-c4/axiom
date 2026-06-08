# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_acos_is_present"
# subject = "cmath.acos"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.acos: api_acos_is_present (surface)."""
import cmath

assert hasattr(cmath, "acos")
print("api_acos_is_present OK")
