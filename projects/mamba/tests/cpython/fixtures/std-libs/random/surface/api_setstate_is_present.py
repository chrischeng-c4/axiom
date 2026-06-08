# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_setstate_is_present"
# subject = "random.setstate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.setstate: api_setstate_is_present (surface)."""
import random

assert hasattr(random, "setstate")
print("api_setstate_is_present OK")
