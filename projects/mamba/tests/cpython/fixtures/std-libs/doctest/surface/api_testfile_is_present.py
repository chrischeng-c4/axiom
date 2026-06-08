# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_testfile_is_present"
# subject = "doctest.testfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.testfile: api_testfile_is_present (surface)."""
import doctest

assert hasattr(doctest, "testfile")
print("api_testfile_is_present OK")
