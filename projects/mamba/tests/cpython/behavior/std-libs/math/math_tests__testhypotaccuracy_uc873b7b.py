# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "math_tests__testhypotaccuracy_uc873b7b"
# subject = "cpython.test_math.MathTests.testHypotAccuracy"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_math.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_math
_suite = unittest.defaultTestLoader.loadTestsFromName("MathTests.testHypotAccuracy", test_math)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MathTests.testHypotAccuracy did not pass"
print("MathTests::testHypotAccuracy: ok")
