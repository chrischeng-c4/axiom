# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "util"
# dimension = "behavior"
# case = "test_find_binding__test_try_except_nested"
# subject = "cpython.test_util.Test_find_binding.test_try_except_nested"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_util.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_util
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_find_binding.test_try_except_nested", test_util)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_find_binding.test_try_except_nested did not pass"
print("Test_find_binding::test_try_except_nested: ok")
