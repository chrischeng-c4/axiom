# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_zero_copy_sendfile__test_file2file_not_supported"
# subject = "cpython.test_shutil.TestZeroCopySendfile.test_file2file_not_supported"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shutil
_suite = unittest.defaultTestLoader.loadTestsFromName("TestZeroCopySendfile.test_file2file_not_supported", test_shutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestZeroCopySendfile.test_file2file_not_supported did not pass"
print("TestZeroCopySendfile::test_file2file_not_supported: ok")
