# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_not_called_after_call_raises"
# subject = "unittest.mock.Mock.assert_not_called"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_not_called: after a call, assert_not_called() raises AssertionError"""
from unittest.mock import MagicMock

m = MagicMock()
m()
_raised = False
try:
    m.assert_not_called()
except AssertionError:
    _raised = True
assert _raised, "expected AssertionError after a call"
print("assert_not_called_after_call_raises OK")
