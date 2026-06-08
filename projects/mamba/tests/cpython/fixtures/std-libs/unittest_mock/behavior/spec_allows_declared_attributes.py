# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "spec_allows_declared_attributes"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: a Mock(spec=cls) permits calling methods the spec class declares, recording the call, while restricting undeclared attributes"""
from unittest.mock import MagicMock


class Svc:
    def get(self, k):
        return k


m = MagicMock(spec=Svc)
m.get("a")
assert m.get.called is True
_raised = False
try:
    m.no_such
except AttributeError:
    _raised = True
assert _raised, "spec restricts undeclared attributes"
print("spec_allows_declared_attributes OK")
