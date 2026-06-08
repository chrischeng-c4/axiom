# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_pairwise_is_present"
# subject = "itertools.pairwise"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.pairwise: api_pairwise_is_present (surface)."""
import itertools

assert hasattr(itertools, "pairwise")
print("api_pairwise_is_present OK")
