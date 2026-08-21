# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "script_helper"
# dimension = "behavior"
# case = "test_script_helper__test_assert_python_not_isolated_when_env_is_required_ucad0a6e"
# subject = "cpython.test_script_helper.TestScriptHelper.test_assert_python_not_isolated_when_env_is_required"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_script_helper.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_script_helper
_suite = unittest.defaultTestLoader.loadTestsFromName("TestScriptHelper.test_assert_python_not_isolated_when_env_is_required", test_script_helper)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestScriptHelper.test_assert_python_not_isolated_when_env_is_required did not pass"
print("TestScriptHelper::test_assert_python_not_isolated_when_env_is_required: ok")
