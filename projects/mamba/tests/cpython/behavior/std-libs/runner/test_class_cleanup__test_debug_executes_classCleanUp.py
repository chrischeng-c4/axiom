# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runner"
# dimension = "behavior"
# case = "test_class_cleanup__test_debug_executes_classCleanUp"
# subject = "cpython.test_runner.TestClassCleanup.test_debug_executes_classCleanUp"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_runner.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_runner
_suite = unittest.defaultTestLoader.loadTestsFromName("TestClassCleanup.test_debug_executes_classCleanUp", test_runner)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestClassCleanup.test_debug_executes_classCleanUp did not pass"
print("TestClassCleanup::test_debug_executes_classCleanUp: ok")
