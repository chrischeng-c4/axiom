# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "gettext_test_case2__test_some_translations_with_context_and_domain"
# subject = "cpython.test_gettext.GettextTestCase2.test_some_translations_with_context_and_domain"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gettext
_suite = unittest.defaultTestLoader.loadTestsFromName("GettextTestCase2.test_some_translations_with_context_and_domain", test_gettext)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GettextTestCase2.test_some_translations_with_context_and_domain did not pass"
print("GettextTestCase2::test_some_translations_with_context_and_domain: ok")
