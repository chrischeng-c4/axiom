# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_temporary_directory__test_cleanup_with_symlink_flags"
# subject = "cpython.test_tempfile.TestTemporaryDirectory.test_cleanup_with_symlink_flags"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTemporaryDirectory.test_cleanup_with_symlink_flags", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTemporaryDirectory.test_cleanup_with_symlink_flags did not pass"
print("TestTemporaryDirectory::test_cleanup_with_symlink_flags: ok")
