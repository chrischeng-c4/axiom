# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_dict_scopes_mutation"
# subject = "unittest.mock.patch.dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch.dict: patch.dict adds keys to a dict inside the block and reverts the dict to its original contents on exit"""
from unittest.mock import patch

d = {"a": 1}
with patch.dict(d, {"b": 2}):
    assert d == {"a": 1, "b": 2}
assert d == {"a": 1}
print("patch_dict_scopes_mutation OK")
