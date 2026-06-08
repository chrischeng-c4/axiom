# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_args_list_records_each_call"
# subject = "unittest.mock.Mock.call_args_list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.call_args_list: call_args_list accumulates one call(...) per invocation in order across repeated calls"""
from unittest.mock import MagicMock, call

m = MagicMock()
m(1)
m(2)
m(3)
assert m.call_args_list == [call(1), call(2), call(3)]
print("call_args_list_records_each_call OK")
