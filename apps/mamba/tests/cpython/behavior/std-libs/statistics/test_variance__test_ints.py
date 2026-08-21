# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_variance__test_ints"
# subject = "cpython.test_statistics.TestVariance.test_ints"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_statistics
_suite = unittest.defaultTestLoader.loadTestsFromName("TestVariance.test_ints", test_statistics)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestVariance.test_ints did not pass"
print("TestVariance::test_ints: ok")
