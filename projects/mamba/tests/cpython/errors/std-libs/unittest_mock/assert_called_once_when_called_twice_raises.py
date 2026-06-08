# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_called_once_when_called_twice_raises"
# subject = "unittest.mock.Mock.assert_called_once"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_called_once: a mock called twice fails assert_called_once() with AssertionError whose message reports it was Called 2 times"""
from unittest.mock import MagicMock

m = MagicMock()
m()
m()
_raised = False
msg = ""
try:
    m.assert_called_once()
except AssertionError as e:
    _raised = True
    msg = str(e)
assert _raised, "expected AssertionError when called twice"
assert "2 times" in msg, msg
print("assert_called_once_when_called_twice_raises OK")
