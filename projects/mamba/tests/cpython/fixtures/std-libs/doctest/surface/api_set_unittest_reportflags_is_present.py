# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_set_unittest_reportflags_is_present"
# subject = "doctest.set_unittest_reportflags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.set_unittest_reportflags: api_set_unittest_reportflags_is_present (surface)."""
import doctest

assert hasattr(doctest, "set_unittest_reportflags")
print("api_set_unittest_reportflags_is_present OK")
