# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_xxsubinterpreters"
# dimension = "behavior"
# case = "is_running_tests__test_subinterpreter_ucb0cb17"
# subject = "cpython.test__xxsubinterpreters.IsRunningTests.test_subinterpreter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__xxsubinterpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test__xxsubinterpreters
_suite = unittest.defaultTestLoader.loadTestsFromName("IsRunningTests.test_subinterpreter", test__xxsubinterpreters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IsRunningTests.test_subinterpreter did not pass"
print("IsRunningTests::test_subinterpreter: ok")
