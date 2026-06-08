# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_except_star__weird_leaf_exceptions__test_reraise_unhashable_leaf"
# subject = "cpython.test_except_star.TestExceptStar_WeirdLeafExceptions.test_reraise_unhashable_leaf"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_except_star
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExceptStar_WeirdLeafExceptions.test_reraise_unhashable_leaf", test_except_star)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExceptStar_WeirdLeafExceptions.test_reraise_unhashable_leaf did not pass"
print("TestExceptStar_WeirdLeafExceptions::test_reraise_unhashable_leaf: ok")
