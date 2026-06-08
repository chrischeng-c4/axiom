# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_fsum_is_present"
# subject = "math.fsum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.fsum: api_fsum_is_present (surface)."""
import math

assert hasattr(math, "fsum")
print("api_fsum_is_present OK")
