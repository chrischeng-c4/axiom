# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "patch_object_missing_attr_raises"
# subject = "unittest.mock.patch.object"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch.object: patch.object(cls, 'no_such_method') as a context manager raises AttributeError because the target attribute does not exist"""
from unittest.mock import patch


class T:
    def method(self) -> int:
        return 1


_raised = False
try:
    with patch.object(T, "no_such_method"):
        pass
except AttributeError:
    _raised = True
assert _raised, "expected AttributeError patching a missing attribute"
print("patch_object_missing_attr_raises OK")
