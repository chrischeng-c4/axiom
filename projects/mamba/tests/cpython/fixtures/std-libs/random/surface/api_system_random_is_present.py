# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_system_random_is_present"
# subject = "random.SystemRandom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.SystemRandom: api_system_random_is_present (surface)."""
import random

assert hasattr(random, "SystemRandom")
print("api_system_random_is_present OK")
