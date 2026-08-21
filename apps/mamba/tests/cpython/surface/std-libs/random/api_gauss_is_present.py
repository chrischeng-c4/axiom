# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_gauss_is_present"
# subject = "random.gauss"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "apps/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.gauss: api_gauss_is_present (surface)."""
import random

assert hasattr(random, "gauss")
print("api_gauss_is_present OK")
