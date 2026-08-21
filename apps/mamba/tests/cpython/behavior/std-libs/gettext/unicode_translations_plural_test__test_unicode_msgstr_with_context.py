# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "unicode_translations_plural_test__test_unicode_msgstr_with_context"
# subject = "cpython.test_gettext.UnicodeTranslationsPluralTest.test_unicode_msgstr_with_context"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gettext
_suite = unittest.defaultTestLoader.loadTestsFromName("UnicodeTranslationsPluralTest.test_unicode_msgstr_with_context", test_gettext)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnicodeTranslationsPluralTest.test_unicode_msgstr_with_context did not pass"
print("UnicodeTranslationsPluralTest::test_unicode_msgstr_with_context: ok")
