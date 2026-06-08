# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "behavior"
# case = "uncompressed_zip_import_test_case__test_checked_hash_based_change_pyc_uc8a7b48"
# subject = "cpython.test_zipimport.UncompressedZipImportTestCase.test_checked_hash_based_change_pyc"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipimport.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zipimport
_suite = unittest.defaultTestLoader.loadTestsFromName("UncompressedZipImportTestCase.test_checked_hash_based_change_pyc", test_zipimport)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UncompressedZipImportTestCase.test_checked_hash_based_change_pyc did not pass"
print("UncompressedZipImportTestCase::test_checked_hash_based_change_pyc: ok")
