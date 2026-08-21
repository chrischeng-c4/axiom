# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "script_helper"
# dimension = "behavior"
# case = "test_script_helper_environment__test_interpreter_requires_environment_true_uc80962f"
# subject = "cpython.test_script_helper.TestScriptHelperEnvironment.test_interpreter_requires_environment_true"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_script_helper.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_script_helper
_suite = unittest.defaultTestLoader.loadTestsFromName("TestScriptHelperEnvironment.test_interpreter_requires_environment_true", test_script_helper)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestScriptHelperEnvironment.test_interpreter_requires_environment_true did not pass"
print("TestScriptHelperEnvironment::test_interpreter_requires_environment_true: ok")
