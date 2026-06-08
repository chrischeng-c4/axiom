# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "security"
# case = "spec_set_blocks_attribute_injection"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: Mock(spec_set=cls) refuses to set an attribute the spec class does not declare, raising AttributeError, so a test double cannot silently grow an unexpected (typo or injected) attribute"""
from unittest.mock import Mock


class Svc:
    def get(self, k):
        return k


m = Mock(spec_set=Svc)
m.get  # a declared attribute is allowed
_raised = False
try:
    m.injected_attr = 123  # a typo / injected attribute
except AttributeError:
    _raised = True
assert _raised, "spec_set must block setting an undeclared attribute"
print("spec_set_blocks_attribute_injection OK")
