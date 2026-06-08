# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "mock_calls_records_nested_children"
# subject = "unittest.mock.Mock.mock_calls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.mock_calls: mock_calls on a parent records child-attribute calls as call.child(...) entries in invocation order"""
from unittest.mock import MagicMock, call

parent = MagicMock()
parent.a()
parent.b(2)
assert parent.mock_calls == [call.a(), call.b(2)]
print("mock_calls_records_nested_children OK")
