# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "side_effect_exception_propagates"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: a mock whose side_effect is an exception instance re-raises that exception when called"""
from unittest.mock import MagicMock

m = MagicMock(side_effect=ValueError("boom"))
_raised = False
try:
    m()
except ValueError as e:
    _raised = True
    assert str(e) == "boom"
assert _raised, "an exception side_effect must be re-raised"
print("side_effect_exception_propagates OK")
