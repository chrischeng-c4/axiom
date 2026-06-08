# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_weibullvariate_is_present"
# subject = "random.weibullvariate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.weibullvariate: api_weibullvariate_is_present (surface)."""
import random

assert hasattr(random, "weibullvariate")
print("api_weibullvariate_is_present OK")
