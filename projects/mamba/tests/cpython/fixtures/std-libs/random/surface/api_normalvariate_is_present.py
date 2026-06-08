# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_normalvariate_is_present"
# subject = "random.normalvariate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.normalvariate: api_normalvariate_is_present (surface)."""
import random

assert hasattr(random, "normalvariate")
print("api_normalvariate_is_present OK")
