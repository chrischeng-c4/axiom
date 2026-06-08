# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_example_is_present"
# subject = "doctest.Example"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.Example: api_example_is_present (surface)."""
import doctest

assert hasattr(doctest, "Example")
print("api_example_is_present OK")
