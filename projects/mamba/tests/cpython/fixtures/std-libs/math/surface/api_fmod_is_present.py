# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "api_fmod_is_present"
# subject = "math.fmod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""math.fmod: api_fmod_is_present (surface)."""
import math

assert hasattr(math, "fmod")
print("api_fmod_is_present OK")
