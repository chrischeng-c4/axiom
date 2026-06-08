# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_filterfalse_is_present"
# subject = "itertools.filterfalse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.filterfalse: api_filterfalse_is_present (surface)."""
import itertools

assert hasattr(itertools, "filterfalse")
print("api_filterfalse_is_present OK")
