# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_groupby_is_present"
# subject = "itertools.groupby"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.groupby: api_groupby_is_present (surface)."""
import itertools

assert hasattr(itertools, "groupby")
print("api_groupby_is_present OK")
