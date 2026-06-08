# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_accumulate_is_present"
# subject = "itertools.accumulate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.accumulate: api_accumulate_is_present (surface)."""
import itertools

assert hasattr(itertools, "accumulate")
print("api_accumulate_is_present OK")
