# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_vonmisesvariate_is_present"
# subject = "random.vonmisesvariate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.vonmisesvariate: api_vonmisesvariate_is_present (surface)."""
import random

assert hasattr(random, "vonmisesvariate")
print("api_vonmisesvariate_is_present OK")
