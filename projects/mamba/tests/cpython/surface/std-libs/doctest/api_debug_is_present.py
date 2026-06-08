# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_debug_is_present"
# subject = "doctest.debug"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.debug: api_debug_is_present (surface)."""
import doctest

assert hasattr(doctest, "debug")
print("api_debug_is_present OK")
