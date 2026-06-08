# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "c_locale_coercion"
# dimension = "behavior"
# case = "locale_configuration_tests__test_external_target_locale_configuration_uccbec26"
# subject = "cpython.test_c_locale_coercion.LocaleConfigurationTests.test_external_target_locale_configuration"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_c_locale_coercion.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_c_locale_coercion
_suite = unittest.defaultTestLoader.loadTestsFromName("LocaleConfigurationTests.test_external_target_locale_configuration", test_c_locale_coercion)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocaleConfigurationTests.test_external_target_locale_configuration did not pass"
print("LocaleConfigurationTests::test_external_target_locale_configuration: ok")
