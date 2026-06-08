# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_text_test_result_is_present"
# subject = "unittest.TextTestResult"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.TextTestResult: api_text_test_result_is_present (surface)."""
import unittest

assert hasattr(unittest, "TextTestResult")
print("api_text_test_result_is_present OK")
