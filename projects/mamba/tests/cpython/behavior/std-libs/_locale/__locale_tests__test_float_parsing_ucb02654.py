# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "behavior"
# case = "__locale_tests__test_float_parsing_ucb02654"
# subject = "cpython.test__locale._LocaleTests.test_float_parsing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test__locale
_suite = unittest.defaultTestLoader.loadTestsFromName("_LocaleTests.test_float_parsing", test__locale)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython _LocaleTests.test_float_parsing did not pass"
print("_LocaleTests::test_float_parsing: ok")
