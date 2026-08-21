# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "plural_forms_internal_test_case__test_gd"
# subject = "cpython.test_gettext.PluralFormsInternalTestCase.test_gd"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gettext
_suite = unittest.defaultTestLoader.loadTestsFromName("PluralFormsInternalTestCase.test_gd", test_gettext)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PluralFormsInternalTestCase.test_gd did not pass"
print("PluralFormsInternalTestCase::test_gd: ok")
