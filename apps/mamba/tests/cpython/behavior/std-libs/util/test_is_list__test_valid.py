# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "util"
# dimension = "behavior"
# case = "test_is_list__test_valid"
# subject = "cpython.test_util.Test_is_list.test_valid"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_util.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_util
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_is_list.test_valid", test_util)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_is_list.test_valid did not pass"
print("Test_is_list::test_valid: ok")
