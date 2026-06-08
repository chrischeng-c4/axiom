# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_nextafter_is_present"
# subject = "math.nextafter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.nextafter: api_nextafter_is_present (surface)."""
import math

assert hasattr(math, "nextafter")
print("api_nextafter_is_present OK")
