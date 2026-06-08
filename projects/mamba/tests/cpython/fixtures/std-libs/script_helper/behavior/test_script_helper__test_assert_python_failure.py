# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "script_helper"
# dimension = "behavior"
# case = "test_script_helper__test_assert_python_failure"
# subject = "cpython.test_script_helper.TestScriptHelper.test_assert_python_failure"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_script_helper.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_script_helper.py::TestScriptHelper::test_assert_python_failure
"""Auto-ported test: TestScriptHelper::test_assert_python_failure (CPython 3.12 oracle)."""


import subprocess
import sys
import os
from test.support import script_helper, requires_subprocess
import unittest
from unittest import mock


'Unittests for test.support.script_helper.  Who tests the test helper?'


# --- test body ---
rc, out, err = script_helper.assert_python_failure('-c', 'sys.exit(0)')

assert 0 != rc
print("TestScriptHelper::test_assert_python_failure: ok")
