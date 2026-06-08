# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_getrandbits_is_present"
# subject = "random.getrandbits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.getrandbits: api_getrandbits_is_present (surface)."""
import random

assert hasattr(random, "getrandbits")
print("api_getrandbits_is_present OK")
