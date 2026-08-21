# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_temporary_directory__test_mkdtemp_failure"
# subject = "cpython.test_tempfile.TestTemporaryDirectory.test_mkdtemp_failure"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTemporaryDirectory.test_mkdtemp_failure", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTemporaryDirectory.test_mkdtemp_failure did not pass"
print("TestTemporaryDirectory::test_mkdtemp_failure: ok")
