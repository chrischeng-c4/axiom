# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_except_star_exception_group_subclass__test_exception_group_subclass_with_bad_split_func"
# subject = "cpython.test_except_star.TestExceptStarExceptionGroupSubclass.test_exception_group_subclass_with_bad_split_func"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_except_star
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExceptStarExceptionGroupSubclass.test_exception_group_subclass_with_bad_split_func", test_except_star)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExceptStarExceptionGroupSubclass.test_exception_group_subclass_with_bad_split_func did not pass"
print("TestExceptStarExceptionGroupSubclass::test_exception_group_subclass_with_bad_split_func: ok")
