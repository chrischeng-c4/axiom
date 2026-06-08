# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_randbytes_is_present"
# subject = "random.randbytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.randbytes: api_randbytes_is_present (surface)."""
import random

assert hasattr(random, "randbytes")
print("api_randbytes_is_present OK")
