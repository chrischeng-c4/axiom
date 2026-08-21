# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unparse"
# dimension = "behavior"
# case = "unparse_test_case__test_constant_tuples"
# subject = "cpython.test_unparse.UnparseTestCase.test_constant_tuples"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_unparse
_suite = unittest.defaultTestLoader.loadTestsFromName("UnparseTestCase.test_constant_tuples", test_unparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnparseTestCase.test_constant_tuples did not pass"
print("UnparseTestCase::test_constant_tuples: ok")
