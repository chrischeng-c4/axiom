# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_tau_is_present"
# subject = "math.tau"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.tau: api_tau_is_present (surface)."""
import math

assert hasattr(math, "tau")
print("api_tau_is_present OK")
