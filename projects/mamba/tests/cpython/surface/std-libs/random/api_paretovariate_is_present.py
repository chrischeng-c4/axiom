# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_paretovariate_is_present"
# subject = "random.paretovariate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.paretovariate: api_paretovariate_is_present (surface)."""
import random

assert hasattr(random, "paretovariate")
print("api_paretovariate_is_present OK")
