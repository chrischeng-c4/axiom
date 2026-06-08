# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_called_with_wrong_args_raises"
# subject = "unittest.mock.Mock.assert_called_with"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_called_with: after one no-arg call, assert_called_with(42) raises AssertionError because the recorded call args do not match"""
from unittest.mock import MagicMock

m = MagicMock()
m()  # recorded call has no arguments
_raised = False
try:
    m.assert_called_with(42)
except AssertionError:
    _raised = True
assert _raised, "expected AssertionError for mismatched call args"
print("assert_called_with_wrong_args_raises OK")
