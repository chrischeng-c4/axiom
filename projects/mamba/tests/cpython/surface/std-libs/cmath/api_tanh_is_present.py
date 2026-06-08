# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_tanh_is_present"
# subject = "cmath.tanh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.tanh: api_tanh_is_present (surface)."""
import cmath

assert hasattr(cmath, "tanh")
print("api_tanh_is_present OK")
