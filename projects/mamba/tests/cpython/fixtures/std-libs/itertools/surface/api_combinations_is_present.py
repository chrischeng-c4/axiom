# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_combinations_is_present"
# subject = "itertools.combinations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.combinations: api_combinations_is_present (surface)."""
import itertools

assert hasattr(itertools, "combinations")
print("api_combinations_is_present OK")
