# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_copy__test_copymode_symlink_to_symlink_wo_lchmod"
# subject = "cpython.test_shutil.TestCopy.test_copymode_symlink_to_symlink_wo_lchmod"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shutil
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCopy.test_copymode_symlink_to_symlink_wo_lchmod", test_shutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCopy.test_copymode_symlink_to_symlink_wo_lchmod did not pass"
print("TestCopy::test_copymode_symlink_to_symlink_wo_lchmod: ok")
