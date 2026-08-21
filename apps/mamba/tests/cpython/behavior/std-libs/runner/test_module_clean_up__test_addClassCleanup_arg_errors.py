# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runner"
# dimension = "behavior"
# case = "test_module_clean_up__test_addClassCleanup_arg_errors"
# subject = "cpython.test_runner.TestModuleCleanUp.test_addClassCleanup_arg_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_runner.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_runner
_suite = unittest.defaultTestLoader.loadTestsFromName("TestModuleCleanUp.test_addClassCleanup_arg_errors", test_runner)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestModuleCleanUp.test_addClassCleanup_arg_errors did not pass"
print("TestModuleCleanUp::test_addClassCleanup_arg_errors: ok")
