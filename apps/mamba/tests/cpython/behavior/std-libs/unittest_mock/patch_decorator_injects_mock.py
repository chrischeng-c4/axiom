# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_decorator_injects_mock"
# subject = "unittest.mock.patch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch: patch used as a decorator injects the mock as a parameter and applies the patch for the duration of the call"""
from unittest.mock import patch
import os


@patch("os.getpid", return_value=4321)
def use(mock_getpid):
    return os.getpid()


assert use() == 4321
assert os.getpid() != 4321  # patch lifted after the call
print("patch_decorator_injects_mock OK")
