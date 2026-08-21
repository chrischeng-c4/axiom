# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "c_locale_coercion"
# dimension = "behavior"
# case = "locale_coercion_tests__test_pythoncoerceclocale_set_to_one_uc8266f5"
# subject = "cpython.test_c_locale_coercion.LocaleCoercionTests.test_PYTHONCOERCECLOCALE_set_to_one"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_c_locale_coercion.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_c_locale_coercion
_suite = unittest.defaultTestLoader.loadTestsFromName("LocaleCoercionTests.test_PYTHONCOERCECLOCALE_set_to_one", test_c_locale_coercion)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocaleCoercionTests.test_PYTHONCOERCECLOCALE_set_to_one did not pass"
print("LocaleCoercionTests::test_PYTHONCOERCECLOCALE_set_to_one: ok")
