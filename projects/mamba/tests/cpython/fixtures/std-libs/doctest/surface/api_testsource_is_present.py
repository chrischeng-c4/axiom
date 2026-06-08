# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_testsource_is_present"
# subject = "doctest.testsource"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.testsource: api_testsource_is_present (surface)."""
import doctest

assert hasattr(doctest, "testsource")
print("api_testsource_is_present OK")
