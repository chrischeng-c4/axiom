# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_distributions__test_betavariate_return_zero"
# subject = "cpython.test_random.TestDistributions.test_betavariate_return_zero"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_random
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDistributions.test_betavariate_return_zero", test_random)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDistributions.test_betavariate_return_zero did not pass"
print("TestDistributions::test_betavariate_return_zero: ok")
