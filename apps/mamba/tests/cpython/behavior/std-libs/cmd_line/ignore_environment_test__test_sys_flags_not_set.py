# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "ignore_environment_test__test_sys_flags_not_set"
# subject = "cpython.test_cmd_line.IgnoreEnvironmentTest.test_sys_flags_not_set"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::IgnoreEnvironmentTest::test_sys_flags_not_set
"""Auto-ported test: IgnoreEnvironmentTest::test_sys_flags_not_set (CPython 3.12 oracle)."""


import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import spawn_python, kill_python, assert_python_ok, assert_python_failure, interpreter_requires_environment


if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

def _kill_python_and_exit_code(p):
    data = kill_python(p)
    returncode = p.wait()
    return (data, returncode)

def tearDownModule():
    support.reap_children()


# --- test body ---
def run_ignoring_vars(predicate, **env_vars):
    code = 'import sys; sys.stderr.write(str(sys.flags)); sys.exit(not ({}))'.format(predicate)
    return assert_python_ok('-E', '-c', code, **env_vars)
expected_outcome = '\n            (sys.flags.debug == sys.flags.optimize ==\n             sys.flags.dont_write_bytecode ==\n             sys.flags.verbose == sys.flags.safe_path == 0)\n        '
run_ignoring_vars(expected_outcome, PYTHONDEBUG='1', PYTHONOPTIMIZE='1', PYTHONDONTWRITEBYTECODE='1', PYTHONVERBOSE='1', PYTHONSAFEPATH='1')
print("IgnoreEnvironmentTest::test_sys_flags_not_set: ok")
