# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_test_suite_is_present"
# subject = "unittest.TestSuite"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.TestSuite: api_test_suite_is_present (surface)."""
import unittest

assert hasattr(unittest, "TestSuite")
print("api_test_suite_is_present OK")
