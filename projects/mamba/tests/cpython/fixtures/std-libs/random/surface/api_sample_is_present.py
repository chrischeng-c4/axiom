# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_sample_is_present"
# subject = "random.sample"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.sample: api_sample_is_present (surface)."""
import random

assert hasattr(random, "sample")
print("api_sample_is_present OK")
