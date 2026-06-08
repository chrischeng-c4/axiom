# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_locale_compiled_uc15c223"
# subject = "cpython.test_re.ReTests.test_locale_compiled"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_re
_suite = unittest.defaultTestLoader.loadTestsFromName("ReTests.test_locale_compiled", test_re)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReTests.test_locale_compiled did not pass"
print("ReTests::test_locale_compiled: ok")
