# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_zip_longest_is_present"
# subject = "itertools.zip_longest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.zip_longest: api_zip_longest_is_present (surface)."""
import itertools

assert hasattr(itertools, "zip_longest")
print("api_zip_longest_is_present OK")
