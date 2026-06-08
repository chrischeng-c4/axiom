# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "api_create_autospec_is_present"
# subject = "unittest.mock.create_autospec"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unittest.mock.create_autospec: api_create_autospec_is_present (surface)."""
import unittest.mock

assert hasattr(unittest.mock, "create_autospec")
print("api_create_autospec_is_present OK")
