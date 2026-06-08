# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "regression_tests__test_long_chain_of_empty_iterables_uc73541c"
# subject = "cpython.test_itertools.RegressionTests.test_long_chain_of_empty_iterables"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_itertools
_suite = unittest.defaultTestLoader.loadTestsFromName("RegressionTests.test_long_chain_of_empty_iterables", test_itertools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RegressionTests.test_long_chain_of_empty_iterables did not pass"
print("RegressionTests::test_long_chain_of_empty_iterables: ok")
