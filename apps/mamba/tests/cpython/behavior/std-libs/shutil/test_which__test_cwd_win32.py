# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_which__test_cwd_win32"
# subject = "cpython.test_shutil.TestWhich.test_cwd_win32"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shutil
_suite = unittest.defaultTestLoader.loadTestsFromName("TestWhich.test_cwd_win32", test_shutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestWhich.test_cwd_win32 did not pass"
print("TestWhich::test_cwd_win32: ok")
