# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "regression_tests__test_issue30347_2_uca11155"
# subject = "cpython.test_itertools.RegressionTests.test_issue30347_2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_itertools
_suite = unittest.defaultTestLoader.loadTestsFromName("RegressionTests.test_issue30347_2", test_itertools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RegressionTests.test_issue30347_2 did not pass"
print("RegressionTests::test_issue30347_2: ok")
