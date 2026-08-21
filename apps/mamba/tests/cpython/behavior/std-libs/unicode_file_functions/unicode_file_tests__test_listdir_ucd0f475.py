# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode_file_functions"
# dimension = "behavior"
# case = "unicode_file_tests__test_listdir_ucd0f475"
# subject = "cpython.test_unicode_file_functions.UnicodeFileTests.test_listdir"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode_file_functions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_unicode_file_functions
_suite = unittest.defaultTestLoader.loadTestsFromName("UnicodeFileTests.test_listdir", test_unicode_file_functions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnicodeFileTests.test_listdir did not pass"
print("UnicodeFileTests::test_listdir: ok")
