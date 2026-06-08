# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_spooled_temporary_file__test_truncate_with_size_parameter"
# subject = "cpython.test_tempfile.TestSpooledTemporaryFile.test_truncate_with_size_parameter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSpooledTemporaryFile.test_truncate_with_size_parameter", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSpooledTemporaryFile.test_truncate_with_size_parameter did not pass"
print("TestSpooledTemporaryFile::test_truncate_with_size_parameter: ok")
