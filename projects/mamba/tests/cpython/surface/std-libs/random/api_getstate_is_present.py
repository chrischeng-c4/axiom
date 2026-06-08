# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_getstate_is_present"
# subject = "random.getstate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.getstate: api_getstate_is_present (surface)."""
import random

assert hasattr(random, "getstate")
print("api_getstate_is_present OK")
