# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_get_default_tempdir__test_no_files_left_behind"
# subject = "cpython.test_tempfile.TestGetDefaultTempdir.test_no_files_left_behind"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestGetDefaultTempdir.test_no_files_left_behind", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestGetDefaultTempdir.test_no_files_left_behind did not pass"
print("TestGetDefaultTempdir::test_no_files_left_behind: ok")
