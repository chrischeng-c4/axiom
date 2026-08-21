# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "plain_mock_len_raises"
# subject = "unittest.mock.Mock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: plain_mock_len_raises (errors)."""
import unittest.mock

_raised = False
try:
    len(unittest.mock.Mock())
except TypeError:
    _raised = True
assert _raised, "plain_mock_len_raises: expected TypeError"
print("plain_mock_len_raises OK")
