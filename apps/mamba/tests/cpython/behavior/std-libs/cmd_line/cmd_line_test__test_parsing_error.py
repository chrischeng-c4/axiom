# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_parsing_error"
# subject = "cpython.test_cmd_line.CmdLineTest.test_parsing_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_parsing_error
"""Auto-ported test: CmdLineTest::test_parsing_error (CPython 3.12 oracle)."""


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
args = [sys.executable, '-I', '--unknown-option']
proc = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
err_msg = 'unknown option --unknown-option\nusage: '

assert proc.stderr.startswith(err_msg)

assert proc.returncode != 0
print("CmdLineTest::test_parsing_error: ok")
