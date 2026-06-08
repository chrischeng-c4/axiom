# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_get_test_case_names_is_present"
# subject = "unittest.getTestCaseNames"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.getTestCaseNames: api_get_test_case_names_is_present (surface)."""
import unittest

assert hasattr(unittest, "getTestCaseNames")
print("api_get_test_case_names_is_present OK")
