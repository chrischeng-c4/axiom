# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "win32_mime_types_test_case__test_registry_parsing_uc2aa4c7"
# subject = "cpython.test_mimetypes.Win32MimeTypesTestCase.test_registry_parsing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mimetypes
_suite = unittest.defaultTestLoader.loadTestsFromName("Win32MimeTypesTestCase.test_registry_parsing", test_mimetypes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Win32MimeTypesTestCase.test_registry_parsing did not pass"
print("Win32MimeTypesTestCase::test_registry_parsing: ok")
