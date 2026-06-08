# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "approx_equal_inexact_test__test_approx_equal_both3"
# subject = "cpython.test_statistics.ApproxEqualInexactTest.test_approx_equal_both3"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_statistics
_suite = unittest.defaultTestLoader.loadTestsFromName("ApproxEqualInexactTest.test_approx_equal_both3", test_statistics)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ApproxEqualInexactTest.test_approx_equal_both3 did not pass"
print("ApproxEqualInexactTest::test_approx_equal_both3: ok")
