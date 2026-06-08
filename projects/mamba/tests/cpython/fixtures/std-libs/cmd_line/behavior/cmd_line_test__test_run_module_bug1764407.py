# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_run_module_bug1764407"
# subject = "cpython.test_cmd_line.CmdLineTest.test_run_module_bug1764407"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_run_module_bug1764407
"""Auto-ported test: CmdLineTest::test_run_module_bug1764407 (CPython 3.12 oracle)."""


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
p = spawn_python('-i', '-m', 'timeit', '-n', '1')
p.stdin.write(b'Timer\n')
p.stdin.write(b'exit()\n')
data = kill_python(p)

assert data.find(b'1 loop') != -1

assert data.find(b'__main__.Timer') != -1
print("CmdLineTest::test_run_module_bug1764407: ok")
