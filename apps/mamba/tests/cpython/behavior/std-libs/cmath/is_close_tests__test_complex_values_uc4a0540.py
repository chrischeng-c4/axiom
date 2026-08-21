# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "is_close_tests__test_complex_values_uc4a0540"
# subject = "cpython.test_cmath.IsCloseTests.test_complex_values"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_cmath
_suite = unittest.defaultTestLoader.loadTestsFromName("IsCloseTests.test_complex_values", test_cmath)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IsCloseTests.test_complex_values did not pass"
print("IsCloseTests::test_complex_values: ok")
