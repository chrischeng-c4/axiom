# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "spec_unknown_attr_raises"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: a Mock(spec=cls) rejects access to an attribute the spec class does not define, raising AttributeError"""
from unittest.mock import MagicMock


class T:
    def method(self) -> int:
        return 1


m = MagicMock(spec=T)
_raised = False
try:
    m.no_such_method
except AttributeError:
    _raised = True
assert _raised, "spec must reject an undeclared attribute"
print("spec_unknown_attr_raises OK")
