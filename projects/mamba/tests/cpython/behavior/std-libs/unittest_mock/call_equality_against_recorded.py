# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_equality_against_recorded"
# subject = "unittest.mock.call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.call: a freshly constructed call(1, 2, k=3) compares equal to the mock's recorded call_args for the same invocation"""
from unittest.mock import MagicMock, call

m = MagicMock()
m(1, 2, k=3)
assert call(1, 2, k=3) == m.call_args
print("call_equality_against_recorded OK")
