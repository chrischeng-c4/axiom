# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__test_sumprod_extended_precision_accuracy_ucbfce04"
# subject = "cpython.test_math.MathTests.test_sumprod_extended_precision_accuracy"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_math
_suite = unittest.defaultTestLoader.loadTestsFromName("MathTests.test_sumprod_extended_precision_accuracy", test_math)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MathTests.test_sumprod_extended_precision_accuracy did not pass"
print("MathTests::test_sumprod_extended_precision_accuracy: ok")
