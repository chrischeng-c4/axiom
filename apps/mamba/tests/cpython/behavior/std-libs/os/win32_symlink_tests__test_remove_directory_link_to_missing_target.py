# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "win32_symlink_tests__test_remove_directory_link_to_missing_target"
# subject = "cpython.test_os.Win32SymlinkTests.test_remove_directory_link_to_missing_target"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("Win32SymlinkTests.test_remove_directory_link_to_missing_target", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Win32SymlinkTests.test_remove_directory_link_to_missing_target did not pass"
print("Win32SymlinkTests::test_remove_directory_link_to_missing_target: ok")
