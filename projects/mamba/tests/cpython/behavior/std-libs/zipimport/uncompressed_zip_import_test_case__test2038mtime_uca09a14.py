# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "behavior"
# case = "uncompressed_zip_import_test_case__test2038mtime_uca09a14"
# subject = "cpython.test_zipimport.UncompressedZipImportTestCase.test2038MTime"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipimport.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zipimport
_suite = unittest.defaultTestLoader.loadTestsFromName("UncompressedZipImportTestCase.test2038MTime", test_zipimport)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UncompressedZipImportTestCase.test2038MTime did not pass"
print("UncompressedZipImportTestCase::test2038MTime: ok")
