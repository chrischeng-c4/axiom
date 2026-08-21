# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "mock_open_is_callable"
# subject = "unittest.mock.mock_open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.mock_open: mock_open_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.mock_open)
print("mock_open_is_callable OK")
