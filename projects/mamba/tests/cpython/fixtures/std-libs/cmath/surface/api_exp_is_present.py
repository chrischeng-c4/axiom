# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_exp_is_present"
# subject = "cmath.exp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.exp: api_exp_is_present (surface)."""
import cmath

assert hasattr(cmath, "exp")
print("api_exp_is_present OK")
