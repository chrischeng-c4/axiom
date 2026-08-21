# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "c_math_tests__testatanhsign_uc9fd966"
# subject = "cpython.test_cmath.CMathTests.testAtanhSign"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_cmath
_suite = unittest.defaultTestLoader.loadTestsFromName("CMathTests.testAtanhSign", test_cmath)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CMathTests.testAtanhSign did not pass"
print("CMathTests::testAtanhSign: ok")
