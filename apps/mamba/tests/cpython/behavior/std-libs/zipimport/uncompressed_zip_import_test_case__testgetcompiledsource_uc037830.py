# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "behavior"
# case = "uncompressed_zip_import_test_case__testgetcompiledsource_uc037830"
# subject = "cpython.test_zipimport.UncompressedZipImportTestCase.testGetCompiledSource"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipimport.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zipimport
_suite = unittest.defaultTestLoader.loadTestsFromName("UncompressedZipImportTestCase.testGetCompiledSource", test_zipimport)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UncompressedZipImportTestCase.testGetCompiledSource did not pass"
print("UncompressedZipImportTestCase::testGetCompiledSource: ok")
