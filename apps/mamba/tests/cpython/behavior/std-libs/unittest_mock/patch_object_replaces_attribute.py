# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_object_replaces_attribute"
# subject = "unittest.mock.patch.object"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch.object: patch.object replaces an existing attribute on an object inside the block and restores it afterwards"""
from unittest.mock import patch


class Svc:
    def get(self):
        return "real"


s = Svc()
with patch.object(Svc, "get", return_value="mocked"):
    assert s.get() == "mocked"
assert s.get() == "real"
print("patch_object_replaces_attribute OK")
