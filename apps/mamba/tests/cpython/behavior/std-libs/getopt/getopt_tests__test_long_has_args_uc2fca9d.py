# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "behavior"
# case = "getopt_tests__test_long_has_args_uc2fca9d"
# subject = "cpython.test_getopt.GetoptTests.test_long_has_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getopt.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_getopt
_suite = unittest.defaultTestLoader.loadTestsFromName("GetoptTests.test_long_has_args", test_getopt)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GetoptTests.test_long_has_args did not pass"
print("GetoptTests::test_long_has_args: ok")
