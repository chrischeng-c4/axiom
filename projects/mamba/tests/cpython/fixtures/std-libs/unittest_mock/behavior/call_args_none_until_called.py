# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_args_none_until_called"
# subject = "unittest.mock.Mock.call_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.call_args: call_args is None before any call and afterwards equals a call(...) capturing the most recent positional and keyword arguments"""
from unittest.mock import MagicMock, call

m = MagicMock()
assert m.call_args is None
m(1, 2, key="value")
assert m.call_args == call(1, 2, key="value")
print("call_args_none_until_called OK")
