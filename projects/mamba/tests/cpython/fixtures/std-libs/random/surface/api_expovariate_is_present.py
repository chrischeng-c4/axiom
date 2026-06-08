# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_expovariate_is_present"
# subject = "random.expovariate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.expovariate: api_expovariate_is_present (surface)."""
import random

assert hasattr(random, "expovariate")
print("api_expovariate_is_present OK")
