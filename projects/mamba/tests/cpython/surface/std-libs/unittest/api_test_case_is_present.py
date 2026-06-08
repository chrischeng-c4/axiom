# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_test_case_is_present"
# subject = "unittest.TestCase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.TestCase: api_test_case_is_present (surface)."""
import unittest

assert hasattr(unittest, "TestCase")
print("api_test_case_is_present OK")
