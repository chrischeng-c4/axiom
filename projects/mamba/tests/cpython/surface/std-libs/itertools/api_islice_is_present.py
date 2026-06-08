# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_islice_is_present"
# subject = "itertools.islice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.islice: api_islice_is_present (surface)."""
import itertools

assert hasattr(itertools, "islice")
print("api_islice_is_present OK")
