# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_random_is_present_2"
# subject = "random.random"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.random: api_random_is_present_2 (surface)."""
import random

assert hasattr(random, "random")
print("api_random_is_present_2 OK")
