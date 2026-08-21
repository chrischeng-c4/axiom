# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_non_interactive_output_buffering"
# subject = "cpython.test_cmd_line.CmdLineTest.test_non_interactive_output_buffering"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_non_interactive_output_buffering
"""Auto-ported test: CmdLineTest::test_non_interactive_output_buffering (CPython 3.12 oracle)."""


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
code = textwrap.dedent('\n            import sys\n            out = sys.stdout\n            print(out.isatty(), out.write_through, out.line_buffering)\n            err = sys.stderr\n            print(err.isatty(), err.write_through, err.line_buffering)\n        ')
args = [sys.executable, '-c', code]
proc = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=True)

assert proc.stdout == 'False False False\nFalse False True\n'
print("CmdLineTest::test_non_interactive_output_buffering: ok")
