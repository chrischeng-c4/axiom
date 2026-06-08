# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "stored_test_zip64_in_small_files__test_bad_zip64_end_of_central_dir"
# subject = "cpython.test_core.StoredTestZip64InSmallFiles.test_bad_zip64_end_of_central_dir"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("StoredTestZip64InSmallFiles.test_bad_zip64_end_of_central_dir", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StoredTestZip64InSmallFiles.test_bad_zip64_end_of_central_dir did not pass"
print("StoredTestZip64InSmallFiles::test_bad_zip64_end_of_central_dir: ok")
