# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_dont_accept_blankline_is_present"
# subject = "doctest.DONT_ACCEPT_BLANKLINE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.DONT_ACCEPT_BLANKLINE: api_dont_accept_blankline_is_present (surface)."""
import doctest

assert hasattr(doctest, "DONT_ACCEPT_BLANKLINE")
print("api_dont_accept_blankline_is_present OK")
