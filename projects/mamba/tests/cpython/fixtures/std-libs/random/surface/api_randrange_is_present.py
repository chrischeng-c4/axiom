# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_randrange_is_present"
# subject = "random.randrange"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.randrange: api_randrange_is_present (surface)."""
import random

assert hasattr(random, "randrange")
print("api_randrange_is_present OK")
