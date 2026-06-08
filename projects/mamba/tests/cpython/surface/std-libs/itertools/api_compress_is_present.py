# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_compress_is_present"
# subject = "itertools.compress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.compress: api_compress_is_present (surface)."""
import itertools

assert hasattr(itertools, "compress")
print("api_compress_is_present OK")
