# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "int"
# dimension = "behavior"
# case = "int_test_cases__test_intconversion_uc8c31fb"
# subject = "cpython.test_int.IntTestCases.test_intconversion"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_int
_suite = unittest.defaultTestLoader.loadTestsFromName("IntTestCases.test_intconversion", test_int)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IntTestCases.test_intconversion did not pass"
print("IntTestCases::test_intconversion: ok")
