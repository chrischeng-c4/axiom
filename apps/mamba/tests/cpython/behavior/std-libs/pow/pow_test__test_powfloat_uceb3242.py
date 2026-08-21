# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pow"
# dimension = "behavior"
# case = "pow_test__test_powfloat_uceb3242"
# subject = "cpython.test_pow.PowTest.test_powfloat"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pow.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pow
_suite = unittest.defaultTestLoader.loadTestsFromName("PowTest.test_powfloat", test_pow)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PowTest.test_powfloat did not pass"
print("PowTest::test_powfloat: ok")
