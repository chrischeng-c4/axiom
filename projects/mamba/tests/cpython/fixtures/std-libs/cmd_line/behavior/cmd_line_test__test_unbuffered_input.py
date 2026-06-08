# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_unbuffered_input"
# subject = "cpython.test_cmd_line.CmdLineTest.test_unbuffered_input"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_unbuffered_input
"""Auto-ported test: CmdLineTest::test_unbuffered_input (CPython 3.12 oracle)."""


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
code = 'import sys; sys.stdout.write(sys.stdin.read(1))'
p = spawn_python('-u', '-c', code)
p.stdin.write(b'x')
p.stdin.flush()
data, rc = _kill_python_and_exit_code(p)

assert rc == 0

assert data.startswith(b'x')
print("CmdLineTest::test_unbuffered_input: ok")
