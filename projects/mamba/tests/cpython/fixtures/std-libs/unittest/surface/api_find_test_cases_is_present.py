# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_find_test_cases_is_present"
# subject = "unittest.findTestCases"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.findTestCases: api_find_test_cases_is_present (surface)."""
import unittest

assert hasattr(unittest, "findTestCases")
print("api_find_test_cases_is_present OK")
