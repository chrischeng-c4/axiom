# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_uniform_is_present"
# subject = "random.uniform"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.uniform: api_uniform_is_present (surface)."""
import random

assert hasattr(random, "uniform")
print("api_uniform_is_present OK")
