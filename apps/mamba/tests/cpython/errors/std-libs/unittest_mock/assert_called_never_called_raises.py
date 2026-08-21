# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_called_never_called_raises"
# subject = "unittest.mock.Mock.assert_called"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_called: assert_called_never_called_raises (errors)."""
import unittest.mock

_raised = False
try:
    unittest.mock.MagicMock().assert_called()
except AssertionError:
    _raised = True
assert _raised, "assert_called_never_called_raises: expected AssertionError"
print("assert_called_never_called_raises OK")
