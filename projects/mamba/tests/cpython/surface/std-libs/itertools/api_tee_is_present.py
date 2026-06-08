# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_tee_is_present"
# subject = "itertools.tee"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.tee: api_tee_is_present (surface)."""
import itertools

assert hasattr(itertools, "tee")
print("api_tee_is_present OK")
