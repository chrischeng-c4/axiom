# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_guess_known_extensions_uc4d7b87"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_guess_known_extensions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mimetypes
_suite = unittest.defaultTestLoader.loadTestsFromName("MimeTypesTestCase.test_guess_known_extensions", test_mimetypes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MimeTypesTestCase.test_guess_known_extensions did not pass"
print("MimeTypesTestCase::test_guess_known_extensions: ok")
