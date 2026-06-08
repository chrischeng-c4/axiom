# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "reset_mock_clears_call_state"
# subject = "unittest.mock.Mock.reset_mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.reset_mock: reset_mock() zeroes call_count, empties call_args_list, and sets call_args back to None"""
from unittest.mock import MagicMock

m = MagicMock()
m(1)
m(2)
m.reset_mock()
assert m.call_count == 0
assert m.call_args_list == []
assert m.call_args is None
print("reset_mock_clears_call_state OK")
