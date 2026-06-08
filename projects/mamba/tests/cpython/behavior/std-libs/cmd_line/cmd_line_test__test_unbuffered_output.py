# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_unbuffered_output"
# subject = "cpython.test_cmd_line.CmdLineTest.test_unbuffered_output"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_unbuffered_output
"""Auto-ported test: CmdLineTest::test_unbuffered_output (CPython 3.12 oracle)."""


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
for stream in ('stdout', 'stderr'):
    code = "import os, sys; sys.%s.buffer.write(b'x'); os._exit(0)" % stream
    rc, out, err = assert_python_ok('-u', '-c', code)
    data = err if stream == 'stderr' else out

    assert data == b'x'
    code = "import os, sys; sys.%s.write('x'); os._exit(0)" % stream
    rc, out, err = assert_python_ok('-u', '-c', code)
    data = err if stream == 'stderr' else out

    assert data == b'x'
print("CmdLineTest::test_unbuffered_output: ok")
