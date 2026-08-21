# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "stored_test_zip64_in_small_files__test_force_zip64"
# subject = "cpython.test_core.StoredTestZip64InSmallFiles.test_force_zip64"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("StoredTestZip64InSmallFiles.test_force_zip64", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StoredTestZip64InSmallFiles.test_force_zip64 did not pass"
print("StoredTestZip64InSmallFiles::test_force_zip64: ok")
