# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_tau_is_present"
# subject = "cmath.tau"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.tau: api_tau_is_present (surface)."""
import cmath

assert hasattr(cmath, "tau")
print("api_tau_is_present OK")
