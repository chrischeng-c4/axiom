# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_triangular_is_present"
# subject = "random.triangular"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.triangular: api_triangular_is_present (surface)."""
import random

assert hasattr(random, "triangular")
print("api_triangular_is_present OK")
