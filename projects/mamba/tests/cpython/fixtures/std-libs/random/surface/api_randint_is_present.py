# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_randint_is_present"
# subject = "random.randint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.randint: api_randint_is_present (surface)."""
import random

assert hasattr(random, "randint")
print("api_randint_is_present OK")
