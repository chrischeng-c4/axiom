# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_testmod_is_present"
# subject = "doctest.testmod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.testmod: api_testmod_is_present (surface)."""
import doctest

assert hasattr(doctest, "testmod")
print("api_testmod_is_present OK")
