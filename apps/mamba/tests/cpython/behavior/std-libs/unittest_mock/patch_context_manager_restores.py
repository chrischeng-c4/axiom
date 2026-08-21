# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_context_manager_restores"
# subject = "unittest.mock.patch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch: patch as a context manager replaces the target inside the block and restores the original afterwards"""
from unittest.mock import patch, MagicMock
import os

original = os.getpid
with patch("os.getpid", return_value=4321) as m:
    assert os.getpid() == 4321
    assert isinstance(m, MagicMock)
assert os.getpid is original
print("patch_context_manager_restores OK")
