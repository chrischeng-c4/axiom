# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_betavariate_is_present"
# subject = "random.betavariate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.betavariate: api_betavariate_is_present (surface)."""
import random

assert hasattr(random, "betavariate")
print("api_betavariate_is_present OK")
