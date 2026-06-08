# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_combinations_with_replacement_is_present"
# subject = "itertools.combinations_with_replacement"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.combinations_with_replacement: api_combinations_with_replacement_is_present (surface)."""
import itertools

assert hasattr(itertools, "combinations_with_replacement")
print("api_combinations_with_replacement_is_present OK")
