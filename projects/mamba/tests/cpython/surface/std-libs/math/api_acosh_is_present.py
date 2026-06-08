# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_acosh_is_present"
# subject = "math.acosh"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.acosh: api_acosh_is_present (surface)."""
import math

assert hasattr(math, "acosh")
print("api_acosh_is_present OK")
