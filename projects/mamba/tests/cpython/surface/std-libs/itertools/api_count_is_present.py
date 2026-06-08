# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_count_is_present"
# subject = "itertools.count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.count: api_count_is_present (surface)."""
import itertools

assert hasattr(itertools, "count")
print("api_count_is_present OK")
