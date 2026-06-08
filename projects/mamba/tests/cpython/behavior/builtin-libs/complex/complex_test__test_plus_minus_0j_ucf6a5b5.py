# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_plus_minus_0j_ucf6a5b5"
# subject = "cpython.test_complex.ComplexTest.test_plus_minus_0j"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_complex
_suite = unittest.defaultTestLoader.loadTestsFromName("ComplexTest.test_plus_minus_0j", test_complex)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ComplexTest.test_plus_minus_0j did not pass"
print("ComplexTest::test_plus_minus_0j: ok")
