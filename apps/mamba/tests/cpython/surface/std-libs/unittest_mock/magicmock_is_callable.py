# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "magicmock_is_callable"
# subject = "unittest.mock.MagicMock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.MagicMock: magicmock_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.MagicMock)
print("magicmock_is_callable OK")
