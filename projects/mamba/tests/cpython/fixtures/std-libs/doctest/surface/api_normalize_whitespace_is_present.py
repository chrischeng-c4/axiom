# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_normalize_whitespace_is_present"
# subject = "doctest.NORMALIZE_WHITESPACE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.NORMALIZE_WHITESPACE: api_normalize_whitespace_is_present (surface)."""
import doctest

assert hasattr(doctest, "NORMALIZE_WHITESPACE")
print("api_normalize_whitespace_is_present OK")
