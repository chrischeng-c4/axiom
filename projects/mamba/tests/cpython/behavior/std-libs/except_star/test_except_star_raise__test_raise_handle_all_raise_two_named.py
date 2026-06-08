# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_except_star_raise__test_raise_handle_all_raise_two_named"
# subject = "cpython.test_except_star.TestExceptStarRaise.test_raise_handle_all_raise_two_named"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_except_star
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExceptStarRaise.test_raise_handle_all_raise_two_named", test_except_star)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExceptStarRaise.test_raise_handle_all_raise_two_named did not pass"
print("TestExceptStarRaise::test_raise_handle_all_raise_two_named: ok")
