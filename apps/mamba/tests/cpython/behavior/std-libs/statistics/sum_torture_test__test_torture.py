# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "sum_torture_test__test_torture"
# subject = "cpython.test_statistics.SumTortureTest.test_torture"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_statistics
_suite = unittest.defaultTestLoader.loadTestsFromName("SumTortureTest.test_torture", test_statistics)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SumTortureTest.test_torture did not pass"
print("SumTortureTest::test_torture: ok")
