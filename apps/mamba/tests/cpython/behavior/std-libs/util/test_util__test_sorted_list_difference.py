# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "util"
# dimension = "behavior"
# case = "test_util__test_sorted_list_difference"
# subject = "cpython.test_util.TestUtil.test_sorted_list_difference"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_util.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_util
_suite = unittest.defaultTestLoader.loadTestsFromName("TestUtil.test_sorted_list_difference", test_util)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestUtil.test_sorted_list_difference did not pass"
print("TestUtil::test_sorted_list_difference: ok")
