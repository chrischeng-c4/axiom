# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_acosh_is_present"
# subject = "cmath.acosh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.acosh: api_acosh_is_present (surface)."""
import cmath

assert hasattr(cmath, "acosh")
print("api_acosh_is_present OK")
