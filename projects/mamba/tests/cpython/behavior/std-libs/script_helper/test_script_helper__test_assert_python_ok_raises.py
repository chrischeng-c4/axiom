# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "script_helper"
# dimension = "behavior"
# case = "test_script_helper__test_assert_python_ok_raises"
# subject = "cpython.test_script_helper.TestScriptHelper.test_assert_python_ok_raises"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_script_helper.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_script_helper.py::TestScriptHelper::test_assert_python_ok_raises
"""Auto-ported test: TestScriptHelper::test_assert_python_ok_raises (CPython 3.12 oracle)."""


import subprocess
import sys
import os
from test.support import script_helper, requires_subprocess
import unittest
from unittest import mock


'Unittests for test.support.script_helper.  Who tests the test helper?'


# --- test body ---
try:
    script_helper.assert_python_ok('-c', 'sys.exit(0)')
    raise AssertionError('expected AssertionError')
except AssertionError as _aR_e:
    import types as _types_aR
    error_context = _types_aR.SimpleNamespace(exception=_aR_e)
error_msg = str(error_context.exception)

assert 'command line:' in error_msg

assert 'sys.exit(0)' in error_msg
print("TestScriptHelper::test_assert_python_ok_raises: ok")
