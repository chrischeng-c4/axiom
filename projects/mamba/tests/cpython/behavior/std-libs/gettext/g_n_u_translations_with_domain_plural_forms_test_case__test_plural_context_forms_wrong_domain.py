# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "g_n_u_translations_with_domain_plural_forms_test_case__test_plural_context_forms_wrong_domain"
# subject = "cpython.test_gettext.GNUTranslationsWithDomainPluralFormsTestCase.test_plural_context_forms_wrong_domain"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gettext
_suite = unittest.defaultTestLoader.loadTestsFromName("GNUTranslationsWithDomainPluralFormsTestCase.test_plural_context_forms_wrong_domain", test_gettext)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GNUTranslationsWithDomainPluralFormsTestCase.test_plural_context_forms_wrong_domain did not pass"
print("GNUTranslationsWithDomainPluralFormsTestCase::test_plural_context_forms_wrong_domain: ok")
