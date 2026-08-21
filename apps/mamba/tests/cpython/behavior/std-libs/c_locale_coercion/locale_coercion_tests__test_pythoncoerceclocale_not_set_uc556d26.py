# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "c_locale_coercion"
# dimension = "behavior"
# case = "locale_coercion_tests__test_pythoncoerceclocale_not_set_uc556d26"
# subject = "cpython.test_c_locale_coercion.LocaleCoercionTests.test_PYTHONCOERCECLOCALE_not_set"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_c_locale_coercion.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_c_locale_coercion
_suite = unittest.defaultTestLoader.loadTestsFromName("LocaleCoercionTests.test_PYTHONCOERCECLOCALE_not_set", test_c_locale_coercion)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocaleCoercionTests.test_PYTHONCOERCECLOCALE_not_set did not pass"
print("LocaleCoercionTests::test_PYTHONCOERCECLOCALE_not_set: ok")
