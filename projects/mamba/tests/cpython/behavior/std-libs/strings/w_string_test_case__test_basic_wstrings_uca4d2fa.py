# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strings"
# dimension = "behavior"
# case = "w_string_test_case__test_basic_wstrings_uca4d2fa"
# subject = "cpython.test_strings.WStringTestCase.test_basic_wstrings"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_strings.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_strings
_suite = unittest.defaultTestLoader.loadTestsFromName("WStringTestCase.test_basic_wstrings", test_strings)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WStringTestCase.test_basic_wstrings did not pass"
print("WStringTestCase::test_basic_wstrings: ok")
