# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "mime_types_test_case__test_filename_with_url_delimiters_ucc09519"
# subject = "cpython.test_mimetypes.MimeTypesTestCase.test_filename_with_url_delimiters"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mimetypes
_suite = unittest.defaultTestLoader.loadTestsFromName("MimeTypesTestCase.test_filename_with_url_delimiters", test_mimetypes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MimeTypesTestCase.test_filename_with_url_delimiters did not pass"
print("MimeTypesTestCase::test_filename_with_url_delimiters: ok")
