# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_batched_is_present"
# subject = "itertools.batched"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.batched: api_batched_is_present (surface)."""
import itertools

assert hasattr(itertools, "batched")
print("api_batched_is_present OK")
