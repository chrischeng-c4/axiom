# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_log_is_present"
# subject = "cmath.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.log: api_log_is_present (surface)."""
import cmath

assert hasattr(cmath, "log")
print("api_log_is_present OK")
