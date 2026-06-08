# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "asyncmock_is_callable"
# subject = "unittest.mock.AsyncMock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.AsyncMock: asyncmock_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.AsyncMock)
print("asyncmock_is_callable OK")
