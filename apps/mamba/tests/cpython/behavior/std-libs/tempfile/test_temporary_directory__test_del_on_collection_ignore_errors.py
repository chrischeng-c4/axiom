# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_temporary_directory__test_del_on_collection_ignore_errors"
# subject = "cpython.test_tempfile.TestTemporaryDirectory.test_del_on_collection_ignore_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTemporaryDirectory.test_del_on_collection_ignore_errors", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTemporaryDirectory.test_del_on_collection_ignore_errors did not pass"
print("TestTemporaryDirectory::test_del_on_collection_ignore_errors: ok")
