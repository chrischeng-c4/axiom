# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_output_newline"
# subject = "cpython.test_cmd_line.CmdLineTest.test_output_newline"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_output_newline
"""Auto-ported test: CmdLineTest::test_output_newline (CPython 3.12 oracle)."""


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
code = 'if 1:\n            import sys\n            print(1)\n            print(2)\n            print(3, file=sys.stderr)\n            print(4, file=sys.stderr)'
rc, out, err = assert_python_ok('-c', code)
if sys.platform == 'win32':

    assert b'1\r\n2\r\n' == out

    assert b'3\r\n4\r\n' == err
else:

    assert b'1\n2\n' == out

    assert b'3\n4\n' == err
print("CmdLineTest::test_output_newline: ok")
