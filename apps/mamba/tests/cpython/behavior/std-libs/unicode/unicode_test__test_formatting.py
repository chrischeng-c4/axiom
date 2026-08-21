# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_formatting"
# subject = "cpython.test_unicode.UnicodeTest.test_formatting"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_unicode
_suite = unittest.defaultTestLoader.loadTestsFromName("UnicodeTest.test_formatting", test_unicode)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnicodeTest.test_formatting did not pass"
print("UnicodeTest::test_formatting: ok")
