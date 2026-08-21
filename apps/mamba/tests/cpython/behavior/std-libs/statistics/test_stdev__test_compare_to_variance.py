# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "test_stdev__test_compare_to_variance"
# subject = "cpython.test_statistics.TestStdev.test_compare_to_variance"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_statistics
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStdev.test_compare_to_variance", test_statistics)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStdev.test_compare_to_variance did not pass"
print("TestStdev::test_compare_to_variance: ok")
