# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "strings"
# dimension = "behavior"
# case = "w_string_array_test_case__test_nonbmp_uc866735"
# subject = "cpython.test_strings.WStringArrayTestCase.test_nonbmp"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_strings.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_strings
_suite = unittest.defaultTestLoader.loadTestsFromName("WStringArrayTestCase.test_nonbmp", test_strings)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython WStringArrayTestCase.test_nonbmp did not pass"
print("WStringArrayTestCase::test_nonbmp: ok")
