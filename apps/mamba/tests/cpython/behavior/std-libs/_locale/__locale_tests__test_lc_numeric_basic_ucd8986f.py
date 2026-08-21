# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "behavior"
# case = "__locale_tests__test_lc_numeric_basic_ucd8986f"
# subject = "cpython.test__locale._LocaleTests.test_lc_numeric_basic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test__locale
_suite = unittest.defaultTestLoader.loadTestsFromName("_LocaleTests.test_lc_numeric_basic", test__locale)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython _LocaleTests.test_lc_numeric_basic did not pass"
print("_LocaleTests::test_lc_numeric_basic: ok")
