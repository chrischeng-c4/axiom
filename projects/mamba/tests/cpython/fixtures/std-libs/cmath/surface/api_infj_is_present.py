# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_infj_is_present"
# subject = "cmath.infj"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.infj: api_infj_is_present (surface)."""
import cmath

assert hasattr(cmath, "infj")
print("api_infj_is_present OK")
