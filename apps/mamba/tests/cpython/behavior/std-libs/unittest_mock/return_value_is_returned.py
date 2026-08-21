# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "return_value_is_returned"
# subject = "unittest.mock.Mock.return_value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.return_value: a mock configured with return_value returns that exact value on every call"""
from unittest.mock import MagicMock

m = MagicMock(return_value=7)
assert m() == 7
assert m(1, 2) == 7
print("return_value_is_returned OK")
