# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "script_helper"
# dimension = "behavior"
# case = "test_script_helper__test_assert_python_isolated_when_env_not_required_uc4f3fda"
# subject = "cpython.test_script_helper.TestScriptHelper.test_assert_python_isolated_when_env_not_required"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_script_helper.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_script_helper
_suite = unittest.defaultTestLoader.loadTestsFromName("TestScriptHelper.test_assert_python_isolated_when_env_not_required", test_script_helper)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestScriptHelper.test_assert_python_isolated_when_env_not_required did not pass"
print("TestScriptHelper::test_assert_python_isolated_when_env_not_required: ok")
