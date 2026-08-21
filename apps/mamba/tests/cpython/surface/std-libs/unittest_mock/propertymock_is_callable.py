# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "surface"
# case = "propertymock_is_callable"
# subject = "unittest.mock.PropertyMock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.mock.PropertyMock: propertymock_is_callable (surface)."""
import unittest.mock

assert callable(unittest.mock.PropertyMock)
print("propertymock_is_callable OK")
