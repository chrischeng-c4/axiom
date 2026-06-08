# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_permutations_is_present"
# subject = "itertools.permutations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.permutations: api_permutations_is_present (surface)."""
import itertools

assert hasattr(itertools, "permutations")
print("api_permutations_is_present OK")
