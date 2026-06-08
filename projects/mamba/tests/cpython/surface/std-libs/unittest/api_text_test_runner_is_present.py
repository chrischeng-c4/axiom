# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "api_text_test_runner_is_present"
# subject = "unittest.TextTestRunner"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.TextTestRunner: api_text_test_runner_is_present (surface)."""
import unittest

assert hasattr(unittest, "TextTestRunner")
print("api_text_test_runner_is_present OK")
