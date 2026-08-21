# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "sealed_mock_new_attr_raises"
# subject = "unittest.mock.seal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testsealable.py"
# status = "filled"
# ///
"""unittest.mock.seal: after seal(mock), touching an unconfigured child attribute raises AttributeError"""
from unittest.mock import MagicMock, seal

m = MagicMock()
m.configured.return_value = 1
seal(m)
assert m.configured() == 1  # configured child still works
_raised = False
try:
    m.unconfigured.deep
except AttributeError:
    _raised = True
assert _raised, "a sealed mock must block a new child attribute"
print("sealed_mock_new_attr_raises OK")
