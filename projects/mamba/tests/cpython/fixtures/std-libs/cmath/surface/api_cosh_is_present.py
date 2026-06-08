# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_cosh_is_present"
# subject = "cmath.cosh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.cosh: api_cosh_is_present (surface)."""
import cmath

assert hasattr(cmath, "cosh")
print("api_cosh_is_present OK")
