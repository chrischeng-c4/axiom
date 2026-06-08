# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "api_default_is_present"
# subject = "unittest.mock.DEFAULT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.mock.DEFAULT: api_default_is_present (surface)."""
import unittest.mock

assert hasattr(unittest.mock, "DEFAULT")
print("api_default_is_present OK")
