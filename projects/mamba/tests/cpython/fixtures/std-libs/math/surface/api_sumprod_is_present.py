# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_sumprod_is_present"
# subject = "math.sumprod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.sumprod: api_sumprod_is_present (surface)."""
import math

assert hasattr(math, "sumprod")
print("api_sumprod_is_present OK")
