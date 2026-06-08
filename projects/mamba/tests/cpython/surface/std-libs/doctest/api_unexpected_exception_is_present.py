# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_unexpected_exception_is_present"
# subject = "doctest.UnexpectedException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.UnexpectedException: api_unexpected_exception_is_present (surface)."""
import doctest

assert hasattr(doctest, "UnexpectedException")
print("api_unexpected_exception_is_present OK")
