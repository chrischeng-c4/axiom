# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "behavior"
# case = "uncompressed_zip_import_test_case__testimport_withstuff_ucd252a1"
# subject = "cpython.test_zipimport.UncompressedZipImportTestCase.testImport_WithStuff"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipimport.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zipimport
_suite = unittest.defaultTestLoader.loadTestsFromName("UncompressedZipImportTestCase.testImport_WithStuff", test_zipimport)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UncompressedZipImportTestCase.testImport_WithStuff did not pass"
print("UncompressedZipImportTestCase::testImport_WithStuff: ok")
