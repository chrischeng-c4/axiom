# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "complex_test__test_richcompare_boundaries_ucf0ad74"
# subject = "cpython.test_complex.ComplexTest.test_richcompare_boundaries"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_complex
_suite = unittest.defaultTestLoader.loadTestsFromName("ComplexTest.test_richcompare_boundaries", test_complex)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ComplexTest.test_richcompare_boundaries did not pass"
print("ComplexTest::test_richcompare_boundaries: ok")
