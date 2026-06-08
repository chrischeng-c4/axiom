# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_function_test_case_is_present"
# subject = "unittest.FunctionTestCase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.FunctionTestCase: api_function_test_case_is_present (surface)."""
import unittest

assert hasattr(unittest, "FunctionTestCase")
print("api_function_test_case_is_present OK")
