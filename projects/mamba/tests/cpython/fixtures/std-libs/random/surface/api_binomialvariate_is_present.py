# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_binomialvariate_is_present"
# subject = "random.binomialvariate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.binomialvariate: api_binomialvariate_is_present (surface)."""
import random

assert hasattr(random, "binomialvariate")
print("api_binomialvariate_is_present OK")
