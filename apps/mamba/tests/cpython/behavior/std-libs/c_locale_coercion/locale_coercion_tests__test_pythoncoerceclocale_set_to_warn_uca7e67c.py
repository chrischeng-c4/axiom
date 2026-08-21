# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "c_locale_coercion"
# dimension = "behavior"
# case = "locale_coercion_tests__test_pythoncoerceclocale_set_to_warn_uca7e67c"
# subject = "cpython.test_c_locale_coercion.LocaleCoercionTests.test_PYTHONCOERCECLOCALE_set_to_warn"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_c_locale_coercion.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_c_locale_coercion
_suite = unittest.defaultTestLoader.loadTestsFromName("LocaleCoercionTests.test_PYTHONCOERCECLOCALE_set_to_warn", test_c_locale_coercion)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocaleCoercionTests.test_PYTHONCOERCECLOCALE_set_to_warn did not pass"
print("LocaleCoercionTests::test_PYTHONCOERCECLOCALE_set_to_warn: ok")
