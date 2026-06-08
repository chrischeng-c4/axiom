# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "unicode_translations_test__test_unicode_msgstr"
# subject = "cpython.test_gettext.UnicodeTranslationsTest.test_unicode_msgstr"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gettext
_suite = unittest.defaultTestLoader.loadTestsFromName("UnicodeTranslationsTest.test_unicode_msgstr", test_gettext)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnicodeTranslationsTest.test_unicode_msgstr did not pass"
print("UnicodeTranslationsTest::test_unicode_msgstr: ok")
