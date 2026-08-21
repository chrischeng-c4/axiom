# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "g_n_u_translations_class_plural_forms_test_case__test_plural_context_forms_null_translations"
# subject = "cpython.test_gettext.GNUTranslationsClassPluralFormsTestCase.test_plural_context_forms_null_translations"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gettext
_suite = unittest.defaultTestLoader.loadTestsFromName("GNUTranslationsClassPluralFormsTestCase.test_plural_context_forms_null_translations", test_gettext)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GNUTranslationsClassPluralFormsTestCase.test_plural_context_forms_null_translations did not pass"
print("GNUTranslationsClassPluralFormsTestCase::test_plural_context_forms_null_translations: ok")
