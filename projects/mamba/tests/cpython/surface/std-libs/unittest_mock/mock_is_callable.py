# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "mock_is_callable"
# subject = "unittest.mock.Mock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.Mock: mock_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.Mock)
print("mock_is_callable OK")
