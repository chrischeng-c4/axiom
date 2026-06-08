# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_cos_is_present"
# subject = "cmath.cos"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.cos: api_cos_is_present (surface)."""
import cmath

assert hasattr(cmath, "cos")
print("api_cos_is_present OK")
