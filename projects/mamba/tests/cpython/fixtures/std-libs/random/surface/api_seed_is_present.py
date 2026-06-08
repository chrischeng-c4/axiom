# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_seed_is_present"
# subject = "random.seed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.seed: api_seed_is_present (surface)."""
import random

assert hasattr(random, "seed")
print("api_seed_is_present OK")
