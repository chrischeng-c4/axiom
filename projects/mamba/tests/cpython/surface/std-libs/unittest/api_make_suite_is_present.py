# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_make_suite_is_present"
# subject = "unittest.makeSuite"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.makeSuite: api_make_suite_is_present (surface)."""
import unittest

assert hasattr(unittest, "makeSuite")
print("api_make_suite_is_present OK")
