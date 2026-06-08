# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_shuffle_is_present"
# subject = "random.shuffle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.shuffle: api_shuffle_is_present (surface)."""
import random

assert hasattr(random, "shuffle")
print("api_shuffle_is_present OK")
