# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "complex"
# dimension = "behavior"
# case = "c_a_p_i_complex_test__test_imagasdouble_uc09e406"
# subject = "cpython.test_complex.CAPIComplexTest.test_imagasdouble"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_complex.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_complex
_suite = unittest.defaultTestLoader.loadTestsFromName("CAPIComplexTest.test_imagasdouble", test_complex)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CAPIComplexTest.test_imagasdouble did not pass"
print("CAPIComplexTest::test_imagasdouble: ok")
