# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runner"
# dimension = "behavior"
# case = "test_module_clean_up__test_with_errors_in_addModuleCleanup_and_setUps"
# subject = "cpython.test_runner.TestModuleCleanUp.test_with_errors_in_addModuleCleanup_and_setUps"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_runner.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_runner
_suite = unittest.defaultTestLoader.loadTestsFromName("TestModuleCleanUp.test_with_errors_in_addModuleCleanup_and_setUps", test_runner)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestModuleCleanUp.test_with_errors_in_addModuleCleanup_and_setUps did not pass"
print("TestModuleCleanUp::test_with_errors_in_addModuleCleanup_and_setUps: ok")
