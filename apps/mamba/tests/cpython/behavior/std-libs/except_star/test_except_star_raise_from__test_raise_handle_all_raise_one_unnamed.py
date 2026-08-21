# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_except_star_raise_from__test_raise_handle_all_raise_one_unnamed"
# subject = "cpython.test_except_star.TestExceptStarRaiseFrom.test_raise_handle_all_raise_one_unnamed"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_except_star
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExceptStarRaiseFrom.test_raise_handle_all_raise_one_unnamed", test_except_star)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExceptStarRaiseFrom.test_raise_handle_all_raise_one_unnamed did not pass"
print("TestExceptStarRaiseFrom::test_raise_handle_all_raise_one_unnamed: ok")
