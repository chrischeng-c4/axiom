# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_dropwhile_is_present"
# subject = "itertools.dropwhile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.dropwhile: api_dropwhile_is_present (surface)."""
import itertools

assert hasattr(itertools, "dropwhile")
print("api_dropwhile_is_present OK")
