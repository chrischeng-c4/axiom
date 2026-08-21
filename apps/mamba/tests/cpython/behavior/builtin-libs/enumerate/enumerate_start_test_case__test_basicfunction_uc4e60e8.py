# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "enumerate_start_test_case__test_basicfunction_uc4e60e8"
# subject = "cpython.test_enumerate.EnumerateStartTestCase.test_basicfunction"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enumerate
_suite = unittest.defaultTestLoader.loadTestsFromName("EnumerateStartTestCase.test_basicfunction", test_enumerate)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EnumerateStartTestCase.test_basicfunction did not pass"
print("EnumerateStartTestCase::test_basicfunction: ok")
